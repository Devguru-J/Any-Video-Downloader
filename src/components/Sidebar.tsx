import { useMemo } from "react";
import clsx from "clsx";
import { activeJobs, useApp } from "../lib/store";

export function Sidebar() {
  const view = useApp((s) => s.view);
  const setView = useApp((s) => s.setView);
  const jobsMap = useApp((s) => s.jobs);
  const activeCount = useMemo(() => activeJobs(jobsMap).length, [jobsMap]);

  return (
    <aside className="w-56 shrink-0 border-r border-neutral-200 bg-neutral-50/60 p-3 backdrop-blur dark:border-neutral-800 dark:bg-neutral-950/60">
      <div className="px-2 py-3 text-sm font-semibold tracking-tight">
        Any Video
      </div>

      <nav className="mt-2 flex flex-col gap-0.5">
        <NavItem
          active={view === "active"}
          onClick={() => setView("active")}
          icon="⬇"
          label="진행 중"
          badge={activeCount > 0 ? activeCount : undefined}
        />
        <NavItem
          active={view === "history"}
          onClick={() => setView("history")}
          icon="📁"
          label="기록"
        />
        <NavItem
          active={view === "settings"}
          onClick={() => setView("settings")}
          icon="⚙"
          label="설정"
        />
      </nav>

      <div className="mt-auto" />
      <AdSlot />
    </aside>
  );
}

function NavItem(props: {
  icon: string;
  label: string;
  active?: boolean;
  onClick: () => void;
  badge?: number;
}) {
  return (
    <button
      onClick={props.onClick}
      className={clsx(
        "group flex items-center gap-2 rounded-md px-2.5 py-1.5 text-sm transition",
        props.active
          ? "bg-white text-neutral-900 shadow-sm dark:bg-neutral-800 dark:text-white"
          : "text-neutral-600 hover:bg-white/70 dark:text-neutral-400 dark:hover:bg-neutral-800/60",
      )}
    >
      <span className="w-4 text-center">{props.icon}</span>
      <span className="flex-1 text-left">{props.label}</span>
      {props.badge != null && (
        <span className="rounded-full bg-neutral-200 px-1.5 text-[11px] font-medium tabular-nums dark:bg-neutral-700">
          {props.badge}
        </span>
      )}
    </button>
  );
}

function AdSlot() {
  // Reserved space; populated when ads ship in v2.0
  return null;
}
