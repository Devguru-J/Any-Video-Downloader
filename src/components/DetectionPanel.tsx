import { useEffect, useMemo, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { ipc } from "../lib/ipc";
import { useApp } from "../lib/store";
import type { VideoFormat } from "../lib/types";
import { formatBytes, formatDuration } from "../lib/format";

export function DetectionPanel() {
  const detection = useApp((s) => s.detection);
  const setDetection = useApp((s) => s.setDetection);
  const defaultFolder = useApp((s) => s.defaultFolder);

  const [formatId, setFormatId] = useState<string | null>(null);
  const [folder, setFolder] = useState(defaultFolder);

  useEffect(() => {
    setFolder(defaultFolder);
  }, [defaultFolder]);

  useEffect(() => {
    if (detection) setFormatId(pickDefault(detection.formats));
  }, [detection]);

  const formats = useMemo(
    () => detection?.formats.slice().sort(byQualityDesc) ?? [],
    [detection],
  );

  if (!detection) return null;

  async function chooseFolder() {
    const picked = await openDialog({ directory: true, multiple: false });
    if (typeof picked === "string") setFolder(picked);
  }

  async function startDownload() {
    if (!detection || !formatId || !folder) return;
    await ipc.startDownload({
      id: crypto.randomUUID(),
      url: detection.url,
      title: detection.title,
      format_id: formatId,
      output_dir: folder,
      thumbnail: detection.thumbnail,
    });
    setDetection(null);
  }

  return (
    <div className="card m-3 p-4">
      <div className="flex items-start gap-3">
        {detection.thumbnail ? (
          <img
            src={detection.thumbnail}
            alt=""
            className="w-28 h-16 rounded object-cover bg-neutral-100 dark:bg-neutral-800"
          />
        ) : (
          <div className="w-28 h-16 rounded bg-neutral-100 dark:bg-neutral-800" />
        )}
        <div className="flex-1 min-w-0">
          <div className="font-medium truncate">{detection.title}</div>
          <div className="text-xs text-neutral-500 mt-0.5">
            {detection.uploader && <>{detection.uploader} · </>}
            {formatDuration(detection.duration)}
          </div>
        </div>
        <button
          className="btn btn-ghost"
          onClick={() => setDetection(null)}
          title="Dismiss"
        >
          ✕
        </button>
      </div>

      <div className="mt-3">
        <label className="text-xs text-neutral-500">Quality</label>
        <select
          className="input mt-1"
          value={formatId ?? ""}
          onChange={(e) => setFormatId(e.target.value)}
        >
          {formats.map((f) => (
            <option key={f.id} value={f.id}>
              {labelFor(f)}
            </option>
          ))}
        </select>
      </div>

      <div className="mt-3">
        <label className="text-xs text-neutral-500">Save to</label>
        <div className="flex gap-2 mt-1">
          <input
            className="input flex-1 truncate"
            readOnly
            value={folder || "Select a folder"}
          />
          <button className="btn btn-ghost" onClick={chooseFolder}>
            Browse…
          </button>
        </div>
      </div>

      <div className="mt-4 flex justify-end">
        <button
          className="btn btn-primary"
          onClick={startDownload}
          disabled={!formatId || !folder}
        >
          Download
        </button>
      </div>
    </div>
  );
}

function labelFor(f: VideoFormat): string {
  const parts: string[] = [];
  if (f.resolution) parts.push(f.resolution);
  if (f.ext) parts.push(f.ext.toUpperCase());
  if (f.fps) parts.push(`${f.fps}fps`);
  if (f.filesize) parts.push(formatBytes(f.filesize));
  if (f.note) parts.push(f.note);
  return parts.join(" · ") || f.id;
}

function pickDefault(formats: VideoFormat[]): string | null {
  if (formats.length === 0) return null;
  const sorted = formats.slice().sort(byQualityDesc);
  const mp4 = sorted.find((f) => f.ext === "mp4");
  return (mp4 ?? sorted[0]).id;
}

function byQualityDesc(a: VideoFormat, b: VideoFormat): number {
  const ah = parseHeight(a.resolution);
  const bh = parseHeight(b.resolution);
  if (ah !== bh) return bh - ah;
  return (b.tbr ?? 0) - (a.tbr ?? 0);
}

function parseHeight(res?: string): number {
  if (!res) return 0;
  const m = /(\d+)$/.exec(res) || /x(\d+)/.exec(res);
  return m ? Number(m[1]) : 0;
}
