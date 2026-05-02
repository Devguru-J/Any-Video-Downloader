use anyhow::{anyhow, Result};
use base64::Engine;
use std::collections::HashSet;
use std::time::Duration;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use url::Url;

/// JS injected into the scraper webview's main world before page scripts run.
/// Captures iframes, video sources, and network requests for media URLs.
/// Communicates back via `document.title` (works cross-origin since the
/// init-script runs in the page's own context).
const SCRAPER_JS: &str = r#"
(function() {
  if (window.__ANY_VIDEO_SCRAPER__) return;
  window.__ANY_VIDEO_SCRAPER__ = true;

  const found = new Set();
  const RE = /https?:\/\/[^\s"'<>(){}]+\.(?:m3u8|mpd|mp4|webm|ts)(?:\?[^\s"'<>(){}]*)?/gi;

  function add(u, kind) {
    if (!u) return;
    const tag = (kind || 'media') + '|' + u;
    if (found.has(tag)) return;
    found.add(tag);
    flush();
  }

  function flush() {
    try {
      const list = Array.from(found).map(s => {
        const i = s.indexOf('|');
        return { kind: s.slice(0, i), url: s.slice(i + 1) };
      });
      const json = JSON.stringify(list);
      const b64 = btoa(unescape(encodeURIComponent(json)));
      // Title channel: Rust polls document.title.
      document.title = 'ANYVIDEO::' + b64;
    } catch (e) { /* ignore */ }
  }

  function scanDom() {
    document.querySelectorAll('iframe').forEach(f => {
      const src = f.src || f.getAttribute('data-src') || f.getAttribute('data-litespeed-src');
      if (src && /^https?:/i.test(src)) add(src, 'iframe');
    });
    document.querySelectorAll('video').forEach(v => {
      if (v.src) add(v.src, 'media');
      v.querySelectorAll('source').forEach(s => {
        if (s.src) add(s.src, 'media');
      });
    });
    // Sniff inline scripts and HTML for direct manifest URLs.
    try {
      const html = document.documentElement.outerHTML;
      let m;
      RE.lastIndex = 0;
      while ((m = RE.exec(html))) add(m[0], 'media');
    } catch (e) { /* ignore */ }
  }

  // Patch fetch + XHR + MediaSource so we catch URLs the page assembles
  // dynamically (very common with JW Player and similar).
  try {
    const origFetch = window.fetch;
    if (origFetch) {
      window.fetch = function(input, init) {
        try {
          const u = typeof input === 'string' ? input : (input && input.url) || '';
          if (/\.(m3u8|mpd|mp4|webm|ts)(\?|$)/i.test(u)) add(u, 'media');
        } catch (e) {}
        return origFetch.apply(this, arguments);
      };
    }
  } catch (e) {}

  try {
    const origOpen = XMLHttpRequest.prototype.open;
    XMLHttpRequest.prototype.open = function(method, url) {
      try {
        if (typeof url === 'string' && /\.(m3u8|mpd|mp4|webm|ts)(\?|$)/i.test(url))
          add(url, 'media');
      } catch (e) {}
      return origOpen.apply(this, arguments);
    };
  } catch (e) {}

  function startObserver() {
    if (!document.body) return;
    new MutationObserver(scanDom).observe(document.body, {
      childList: true, subtree: true, attributes: true,
      attributeFilter: ['src', 'data-src']
    });
  }

  if (document.body) startObserver();
  else document.addEventListener('DOMContentLoaded', startObserver);

  setInterval(scanDom, 1500);
  scanDom();
})();
"#;

#[derive(Debug, Clone)]
pub struct ScrapedUrl {
    pub kind: String, // "iframe" | "media"
    pub url: String,
}

/// Open a hidden webview that loads `url`, lets the real browser engine pass
/// any Cloudflare JS challenge, and gathers iframe/media URLs the page
/// references. If the challenge appears to need user interaction, the window
/// becomes visible.
///
/// `max_wait` caps the total wait time. Returns whatever was captured by
/// then; an empty Vec means nothing was found.
pub async fn scrape(
    app: &AppHandle,
    url: &str,
    max_wait: Duration,
) -> Result<Vec<ScrapedUrl>> {
    let parsed: Url = url
        .parse()
        .map_err(|e| anyhow!("invalid URL `{url}`: {e}"))?;

    let label = format!(
        "scraper-{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("x")
    );

    let window = WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::External(parsed),
    )
    .title("any-video scanner")
    .visible(false)
    .inner_size(1100.0, 720.0)
    .initialization_script(SCRAPER_JS)
    .build()?;

    let start = tokio::time::Instant::now();
    let mut last: Vec<ScrapedUrl> = Vec::new();
    let mut shown = false;
    let mut last_progress = start;

    while start.elapsed() < max_wait {
        tokio::time::sleep(Duration::from_millis(700)).await;

        let title = window.title().unwrap_or_default();

        // Reveal the window if Cloudflare seems to be holding us up so the
        // user can solve any interactive challenge themselves.
        if !shown
            && (title.contains("Just a moment")
                || title.contains("Cloudflare")
                || title.contains("Attention Required"))
            && start.elapsed() > Duration::from_secs(4)
        {
            let _ = window.show();
            let _ = window.set_focus();
            shown = true;
        }

        if let Some(b64) = title.strip_prefix("ANYVIDEO::") {
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                if let Ok(json) = std::str::from_utf8(&bytes) {
                    if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
                        let parsed: Vec<ScrapedUrl> = items
                            .into_iter()
                            .filter_map(|v| {
                                Some(ScrapedUrl {
                                    kind: v.get("kind")?.as_str()?.to_string(),
                                    url: v.get("url")?.as_str()?.to_string(),
                                })
                            })
                            .collect();

                        if parsed.len() > last.len() {
                            last_progress = tokio::time::Instant::now();
                        }
                        last = parsed;
                    }
                }
            }
        }

        // If we've already captured something AND nothing new has appeared
        // for a couple seconds, stop early.
        if !last.is_empty() && last_progress.elapsed() > Duration::from_secs(3) {
            break;
        }
    }

    let _ = window.close();

    // Dedup by URL while preserving order.
    let mut seen = HashSet::new();
    let result: Vec<ScrapedUrl> = last
        .into_iter()
        .filter(|x| seen.insert(x.url.clone()))
        .collect();
    Ok(result)
}

/// Heuristic: rank candidate URLs so the most likely "real video" is tried
/// first. Direct manifests > player iframes > everything else.
pub fn rank(candidates: &mut Vec<ScrapedUrl>) {
    candidates.sort_by_key(|c| {
        let u = c.url.to_lowercase();
        let media_score = if u.contains(".m3u8") {
            0
        } else if u.contains(".mpd") {
            1
        } else if u.contains(".mp4") {
            2
        } else if u.contains(".webm") {
            3
        } else {
            10
        };
        let kind_score = if c.kind == "media" { 0 } else { 5 };
        // Penalise obvious garbage (analytics, ads, social embeds).
        let noise = if u.contains("googletag")
            || u.contains("doubleclick")
            || u.contains("facebook.com")
            || u.contains("twitter.com")
            || u.contains("disqus")
            || u.contains("tracker")
            || u.contains("analytics")
        {
            100
        } else {
            0
        };
        media_score + kind_score + noise
    });
}
