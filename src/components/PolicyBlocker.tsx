import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { exit } from "@tauri-apps/plugin-process";
import { useState } from "react";

export function PolicyBlocker({ reason }: { reason: string }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function update() {
    setBusy(true);
    setError(null);
    try {
      const u = await check();
      if (u) {
        await u.downloadAndInstall();
        await relaunch();
      } else {
        setError("지금은 사용 가능한 업데이트가 없습니다. 잠시 후 다시 시도해 주세요.");
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex h-full items-center justify-center p-8">
      <div className="card max-w-md w-full p-8 text-center">
        <div className="text-3xl mb-3">⬆</div>
        <h1 className="text-lg font-semibold">업데이트가 필요합니다</h1>
        <p className="mt-2 text-sm text-neutral-600 dark:text-neutral-400">
          {reason}
        </p>
        {error && (
          <p className="mt-3 text-xs text-red-600 dark:text-red-400">{error}</p>
        )}
        <div className="mt-6 flex gap-2 justify-center">
          <button className="btn btn-ghost" onClick={() => exit(0)}>
            종료
          </button>
          <button
            className="btn btn-primary"
            disabled={busy}
            onClick={update}
          >
            {busy ? "업데이트 중…" : "지금 업데이트"}
          </button>
        </div>
      </div>
    </div>
  );
}
