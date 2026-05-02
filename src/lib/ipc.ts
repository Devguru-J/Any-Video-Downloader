import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BrowserCookies,
  DownloadJob,
  JobState,
  UpdatePolicy,
  VideoMeta,
} from "./types";

export const ipc = {
  probeUrl: (url: string, cookiesFromBrowser: BrowserCookies = "none") =>
    invoke<VideoMeta>("probe_url", { url, cookiesFromBrowser }),
  startDownload: (job: DownloadJob, cookiesFromBrowser: BrowserCookies = "none") =>
    invoke<string>("start_download", { job, cookiesFromBrowser }),
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
