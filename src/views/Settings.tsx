import { useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { useEffect } from "react";
import { useApp } from "../lib/store";

export function SettingsView() {
  const defaultFolder = useApp((s) => s.defaultFolder);
  const setDefaultFolder = useApp((s) => s.setDefaultFolder);

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
        setUpdateMsg(`v${u.version} available`);
        setUpdateState("downloading");
        await u.downloadAndInstall();
        await relaunch();
      } else {
        setUpdateState("none");
        setUpdateMsg("You're on the latest version");
      }
    } catch (e) {
      setUpdateState("error");
      setUpdateMsg(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="max-w-xl mx-auto p-6 space-y-6">
      <Section title="Downloads">
        <Row label="Default folder">
          <div className="flex gap-2 w-full">
            <input
              className="input flex-1 truncate"
              readOnly
              value={defaultFolder || "Not set"}
            />
            <button className="btn btn-ghost" onClick={pickFolder}>
              Browse…
            </button>
          </div>
        </Row>
      </Section>

      <Section title="Updates">
        <Row label="Current version">
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
                ? "Checking…"
                : updateState === "downloading"
                  ? "Downloading…"
                  : "Check for updates"}
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

      <Section title="About">
        <Row label="App">Any Video Downloader</Row>
        <Row label="Engine">yt-dlp + ffmpeg (bundled)</Row>
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
