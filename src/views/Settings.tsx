import { useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { useEffect } from "react";
import { useApp } from "../lib/store";
import type { BrowserCookies } from "../lib/types";

export function SettingsView() {
  const defaultFolder = useApp((s) => s.defaultFolder);
  const setDefaultFolder = useApp((s) => s.setDefaultFolder);
  const cookiesFromBrowser = useApp((s) => s.cookiesFromBrowser);
  const setCookiesFromBrowser = useApp((s) => s.setCookiesFromBrowser);

  const [version, setVersion] = useState<string>("…");
  const [updateState, setUpdateState] = useState<
    "idle" | "checking" | "available" | "none" | "error" | "downloading"
  >("idle");
  const [updateMsg, setUpdateMsg] = useState<string>("");

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion("?"));
  }, []);

  async function pickFolder() {
    const picked = await openDialog({ directory: true, multiple: false });
    if (typeof picked === "string") setDefaultFolder(picked);
  }

  async function checkUpdates() {
    setUpdateState("checking");
    setUpdateMsg("");
    try {
      const u = await check();
      if (u) {
        setUpdateState("available");
        setUpdateMsg(`v${u.version} 업데이트가 있습니다`);
        setUpdateState("downloading");
        await u.downloadAndInstall();
        await relaunch();
      } else {
        setUpdateState("none");
        setUpdateMsg("최신 버전을 사용 중입니다");
      }
    } catch (e) {
      setUpdateState("error");
      setUpdateMsg(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="max-w-xl mx-auto p-6 space-y-6">
      <Section title="다운로드">
        <Row label="기본 폴더">
          <div className="flex gap-2 w-full">
            <input
              className="input flex-1 truncate"
              readOnly
              value={defaultFolder || "설정되지 않음"}
            />
            <button className="btn btn-ghost" onClick={pickFolder}>
              찾아보기…
            </button>
          </div>
        </Row>
      </Section>

      <Section title="고급">
        <Row label="브라우저 쿠키 사용">
          <select
            className="input"
            value={cookiesFromBrowser}
            onChange={(e) =>
              setCookiesFromBrowser(e.target.value as BrowserCookies)
            }
          >
            <option value="none">사용 안 함</option>
            <option value="chrome">Chrome</option>
            <option value="safari">Safari</option>
            <option value="firefox">Firefox</option>
            <option value="edge">Edge</option>
            <option value="brave">Brave</option>
          </select>
        </Row>
        <p className="text-xs text-neutral-500 leading-relaxed">
          Cloudflare 보호 사이트나 로그인이 필요한 사이트에서 다운로드가 막히면
          선택한 브라우저로 그 사이트에 한 번 접속해 챌린지를 통과시킨 뒤 여기
          옵션을 켜세요. 그 브라우저의 쿠키를 빌려 같은 세션으로 접근합니다.
          (브라우저는 종료된 상태여야 쿠키 DB를 잠그지 않습니다.)
        </p>
      </Section>

      <Section title="업데이트">
        <Row label="현재 버전">
          <span className="text-sm tabular-nums">v{version}</span>
        </Row>
        <Row label="">
          <div className="flex items-center gap-2 w-full">
            <button
              className="btn btn-ghost"
              onClick={checkUpdates}
              disabled={
                updateState === "checking" || updateState === "downloading"
              }
            >
              {updateState === "checking"
                ? "확인 중…"
                : updateState === "downloading"
                  ? "다운로드 중…"
                  : "업데이트 확인"}
            </button>
            {updateMsg && (
              <span
                className={
                  updateState === "error"
                    ? "text-xs text-red-600 dark:text-red-400"
                    : "text-xs text-neutral-500"
                }
              >
                {updateMsg}
              </span>
            )}
          </div>
        </Row>
      </Section>

      <Section title="정보">
        <Row label="앱">Any Video Downloader</Row>
        <Row label="엔진">yt-dlp + ffmpeg (내장)</Row>
      </Section>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section>
      <h2 className="text-xs uppercase tracking-wider text-neutral-500 mb-2">
        {title}
      </h2>
      <div className="card p-4 space-y-3">{children}</div>
    </section>
  );
}

function Row({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center gap-4">
      <div className="w-32 shrink-0 text-sm text-neutral-600 dark:text-neutral-400">
        {label}
      </div>
      <div className="flex-1 min-w-0">{children}</div>
    </div>
  );
}
