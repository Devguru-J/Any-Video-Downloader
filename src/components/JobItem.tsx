import { ipc } from "../lib/ipc";
import type { JobState } from "../lib/types";
import { formatBytes, formatEta, formatSpeed } from "../lib/format";

export function JobItem({ job }: { job: JobState }) {
  const s = job.status;
  const pct =
    s.kind === "running" && s.total > 0
      ? Math.min(100, (s.downloaded / s.total) * 100)
      : s.kind === "done"
        ? 100
        : 0;

  return (
    <div className="card mx-3 mb-2 p-3">
      <div className="flex items-center gap-3">
        {job.thumbnail ? (
          <img
            src={job.thumbnail}
            alt=""
            className="w-16 h-10 rounded object-cover bg-neutral-100 dark:bg-neutral-800"
          />
        ) : (
          <div className="w-16 h-10 rounded bg-neutral-100 dark:bg-neutral-800" />
        )}
        <div className="flex-1 min-w-0">
          <div className="text-sm truncate">{job.title}</div>
          <div className="mt-1 h-1 rounded-full bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
            <div
              className={
                s.kind === "error"
                  ? "h-full bg-red-500"
                  : s.kind === "cancelled"
                    ? "h-full bg-neutral-400"
                    : "h-full bg-neutral-900 dark:bg-white"
              }
              style={{ width: `${pct}%` }}
            />
          </div>
          <div className="mt-1 text-[11px] text-neutral-500 tabular-nums flex gap-2">
            <StatusLine job={job} />
          </div>
        </div>
        <Actions job={job} />
      </div>
    </div>
  );
}

function StatusLine({ job }: { job: JobState }) {
  const s = job.status;
  switch (s.kind) {
    case "queued":
      return <span>Queued</span>;
    case "running":
      return (
        <>
          <span>{formatBytes(s.downloaded)}</span>
          {s.total > 0 && <span>/ {formatBytes(s.total)}</span>}
          <span>· {formatSpeed(s.speed)}</span>
          {s.eta > 0 && <span>· {formatEta(s.eta)}</span>}
        </>
      );
    case "done":
      return <span className="text-green-600 dark:text-green-400">✓ Done</span>;
    case "error":
      return <span className="text-red-600 dark:text-red-400" title={s.message}>Failed</span>;
    case "cancelled":
      return <span>Cancelled</span>;
  }
}

function Actions({ job }: { job: JobState }) {
  const s = job.status;
  if (s.kind === "running" || s.kind === "queued") {
    return (
      <button
        className="btn btn-ghost"
        onClick={() => ipc.cancelDownload(job.id)}
        title="Cancel"
      >
        ✕
      </button>
    );
  }
  if (s.kind === "done") {
    return (
      <button
        className="btn btn-ghost"
        onClick={() => ipc.openInFinder(s.path)}
        title="Reveal in Finder"
      >
        ↗
      </button>
    );
  }
  return null;
}
