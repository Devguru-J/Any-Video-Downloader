import { useState } from "react";
import { ipc } from "../lib/ipc";
import { useApp } from "../lib/store";

export function UrlBar() {
  const [url, setUrl] = useState("");
  const detecting = useApp((s) => s.detecting);
  const setDetecting = useApp((s) => s.setDetecting);
  const setDetection = useApp((s) => s.setDetection);

  async function detect() {
    if (!url.trim()) return;
    setDetecting(true);
    setDetection(null);
    try {
      const meta = await ipc.probeUrl(url.trim());
      setDetection(meta);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      alert(`Couldn't detect a video on this page.\n\n${msg}`);
    } finally {
      setDetecting(false);
    }
  }

  function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") detect();
  }

  async function pasteAndDetect() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        setUrl(text);
        // small defer so input updates before detect runs
        setTimeout(() => detect(), 0);
      }
    } catch {
      // ignore — user can type/paste manually
    }
  }

  return (
    <div className="flex items-center gap-2 p-3 border-b border-neutral-200 dark:border-neutral-800">
      <span className="px-2 text-neutral-400">🔗</span>
      <input
        className="input flex-1"
        placeholder="Paste video page URL"
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        onKeyDown={onKeyDown}
        disabled={detecting}
      />
      <button
        className="btn btn-ghost"
        onClick={pasteAndDetect}
        disabled={detecting}
        title="Paste from clipboard and detect"
      >
        Paste
      </button>
      <button
        className="btn btn-primary"
        onClick={detect}
        disabled={detecting || !url.trim()}
      >
        {detecting ? "Detecting…" : "Detect"}
      </button>
    </div>
  );
}
