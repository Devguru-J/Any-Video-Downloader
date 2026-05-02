import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloadJob, JobState, UpdatePolicy, VideoMeta } from "./types";

export const ipc = {
  probeUrl: (url: string) => invoke<VideoMeta>("probe_url", { url }),
  startDownload: (job: DownloadJob) =>
    invoke<string>("start_download", { job }),
  cancelDownload: (jobId: string) =>
    invoke<void>("cancel_download", { jobId }),
  openInFinder: (path: string) => invoke<void>("open_in_finder", { path }),
  checkPolicy: () => invoke<UpdatePolicy>("check_policy"),
  listJobs: () => invoke<JobState[]>("list_jobs"),
  clearHistory: () => invoke<void>("clear_history"),
};

export async function onJobUpdate(
  cb: (state: JobState) => void,
): Promise<UnlistenFn> {
  return listen<JobState>("download://progress", (e) => cb(e.payload));
}
