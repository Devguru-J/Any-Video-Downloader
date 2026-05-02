import { useMemo } from "react";
import { UrlBar } from "../components/UrlBar";
import { DetectionPanel } from "../components/DetectionPanel";
import { JobItem } from "../components/JobItem";
import { activeJobs, useApp } from "../lib/store";

export function ActiveView() {
  const jobsMap = useApp((s) => s.jobs);
  const jobs = useMemo(() => activeJobs(jobsMap), [jobsMap]);

  return (
    <div className="flex flex-col h-full">
      <UrlBar />
      <DetectionPanel />
      <div className="flex-1 overflow-y-auto py-2">
        {jobs.length === 0 ? (
          <div className="px-6 py-12 text-center text-sm text-neutral-500">
            No active downloads. Paste a URL above to get started.
          </div>
        ) : (
          jobs.map((j) => <JobItem key={j.id} job={j} />)
        )}
      </div>
    </div>
  );
}
