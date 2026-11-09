import type { Run, RunStatus, RunTrigger, StepStatus } from "@/api/types"

/**
 * How a status looks. Class names are written out in full on purpose: Tailwind finds classes by
 * scanning the source, so a class built at runtime ("text-status-" + status) would never be generated.
 */
export type StatusStyle = {
  label: string
  /** text + tinted background, for badges */
  badge: string
  /** solid colour, for shapes only (dots, bars), never for text */
  mark: string
  /** filled or hollow dot, so status isn't shown by colour alone */
  filled: boolean
}

export const runStatusStyle: Record<RunStatus, StatusStyle> = {
  succeeded: {
    label: "Succeeded",
    badge: "text-status-succeeded bg-status-succeeded-bg",
    mark: "bg-status-succeeded-mark",
    filled: true,
  },
  failed: {
    label: "Failed",
    badge: "text-status-failed bg-status-failed-bg",
    mark: "bg-status-failed-mark",
    filled: true,
  },
  running: {
    label: "Running",
    badge: "text-status-running bg-status-running-bg",
    mark: "bg-status-running-mark",
    filled: false,
  },
  aborted: {
    label: "Aborted",
    badge: "text-status-aborted bg-status-aborted-bg",
    mark: "bg-status-aborted-mark",
    filled: false,
  },
}

/** All run statuses, in the order filters show them. */
export const runStatuses = Object.keys(runStatusStyle) as RunStatus[]

/** Narrows an untrusted string (e.g. from the URL) to a RunStatus. */
export function isRunStatus(value: string | null): value is RunStatus {
  return value !== null && value in runStatusStyle
}

/** Steps use the same three styles as runs. */
export const stepStatusStyle: Record<StepStatus, StatusStyle> = {
  running: runStatusStyle.running,
  succeeded: runStatusStyle.succeeded,
  failed: runStatusStyle.failed,
}

export const triggerLabel: Record<RunTrigger, string> = {
  external: "External",
  schedule: "Schedule",
  ui: "UI",
  missed: "Missed",
}

/** Short text shown next to a Failed badge, or null when there's nothing to add. */
export function failureText(run: Run): string | null {
  switch (run.failure_reason) {
    case "reported":
      return run.exit_code !== null ? `exit code ${run.exit_code}` : null
    case "heartbeat_timeout":
      return "heartbeat timeout"
    case "missed":
      return "missed"
    case null:
      return null
  }
}
