import { useEffect } from "react";
import { Sidebar } from "./components/Sidebar";
import { ActiveView } from "./views/Active";
import { HistoryView } from "./views/History";
import { SettingsView } from "./views/Settings";
import { PolicyBlocker } from "./components/PolicyBlocker";
import { useApp } from "./lib/store";
import { ipc, onJobUpdate } from "./lib/ipc";

export default function App() {
  const view = useApp((s) => s.view);
  const policyBlocked = useApp((s) => s.policyBlocked);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    (async () => {
      try {
        const jobs = await ipc.listJobs();
        if (!cancelled) useApp.getState().setJobs(jobs);
      } catch {
        /* first-run */
      }

      unlisten = await onJobUpdate((j) => useApp.getState().upsertJob(j));

      try {
        const policy = await ipc.checkPolicy();
        const current = (window as unknown as { __APP_VERSION__?: string })
          .__APP_VERSION__ ?? "0.0.0";
        if (!cancelled && compare(current, policy.min_required_version) < 0) {
          useApp.getState().setPolicyBlocked({ reason: policy.message });
        }
      } catch {
        /* offline */
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
    // Run exactly once on mount; store actions are accessed via getState().
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (policyBlocked) return <PolicyBlocker reason={policyBlocked.reason} />;

  return (
    <div className="flex h-full">
      <Sidebar />
      <main className="flex-1 overflow-y-auto">
        {view === "active" && <ActiveView />}
        {view === "history" && <HistoryView />}
        {view === "settings" && <SettingsView />}
      </main>
    </div>
  );
}

function compare(a: string, b: string): number {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (x !== y) return x - y;
  }
  return 0;
}
