import { useMemo } from "react";
import { JobItem } from "../components/JobItem";
import { historyJobs, useApp } from "../lib/store";
import { ipc } from "../lib/ipc";

export function HistoryView() {
  const jobsMap = useApp((s) => s.jobs);
  const jobs = useMemo(
    () =>
      historyJobs(jobsMap).sort(
        (a, b) => (b.ended_at ?? 0) - (a.ended_at ?? 0),
      ),
    [jobsMap],
  );
  const setJobs = useApp((s) => s.setJobs);

  async function clear() {
    if (!confirm("Clear download history? Files on disk are not removed.")) return;
    await ipc.clearHistory();
    setJobs([]);
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-neutral-200 dark:border-neutral-800">
        <div className="text-sm font-medium">History</div>
        <button className="btn btn-ghost" onClick={clear} disabled={jobs.length === 0}>
          Clear
        </button>
      </div>
      <div className="flex-1 overflow-y-auto py-2">
        {jobs.length === 0 ? (
          <div className="px-6 py-12 text-center text-sm text-neutral-500">
            No completed downloads yet.
          </div>
        ) : (
          jobs.map((j) => <JobItem key={j.id} job={j} />)
        )}
      </div>
    </div>
  );
}
