import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { BrowserCookies, JobState, VideoMeta } from "./types";

type View = "active" | "history" | "settings";

interface AppState {
  view: View;
  setView: (v: View) => void;

  jobs: Record<string, JobState>;
  upsertJob: (j: JobState) => void;
  removeJob: (id: string) => void;
  setJobs: (jobs: JobState[]) => void;

  detection: VideoMeta | null;
  detecting: boolean;
  setDetection: (m: VideoMeta | null) => void;
  setDetecting: (b: boolean) => void;

  defaultFolder: string;
  setDefaultFolder: (p: string) => void;

  cookiesFromBrowser: BrowserCookies;
  setCookiesFromBrowser: (b: BrowserCookies) => void;

  policyBlocked: { reason: string } | null;
  setPolicyBlocked: (b: { reason: string } | null) => void;
}

export const useApp = create<AppState>()(
  persist(
    (set) => ({
      view: "active",
      setView: (view) => set({ view }),

      jobs: {},
      upsertJob: (j) =>
        set((s) => ({ jobs: { ...s.jobs, [j.id]: j } })),
      removeJob: (id) =>
        set((s) => {
          const { [id]: _, ...rest } = s.jobs;
          return { jobs: rest };
        }),
      setJobs: (jobs) =>
        set({ jobs: Object.fromEntries(jobs.map((j) => [j.id, j])) }),

      detection: null,
      detecting: false,
      setDetection: (detection) => set({ detection }),
      setDetecting: (detecting) => set({ detecting }),

      defaultFolder: "",
      setDefaultFolder: (defaultFolder) => set({ defaultFolder }),

      cookiesFromBrowser: "none",
      setCookiesFromBrowser: (cookiesFromBrowser) => set({ cookiesFromBrowser }),

      policyBlocked: null,
      setPolicyBlocked: (policyBlocked) => set({ policyBlocked }),
    }),
    {
      name: "any-video:settings",
      partialize: (s) => ({
        defaultFolder: s.defaultFolder,
        cookiesFromBrowser: s.cookiesFromBrowser,
      }),
    },
  ),
);

export const activeJobs = (jobs: Record<string, JobState>) =>
  Object.values(jobs).filter(
    (j) => j.status.kind === "queued" || j.status.kind === "running",
  );

export const historyJobs = (jobs: Record<string, JobState>) =>
  Object.values(jobs).filter(
    (j) =>
      j.status.kind === "done" ||
      j.status.kind === "error" ||
      j.status.kind === "cancelled",
  );
