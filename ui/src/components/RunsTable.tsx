import type { Job, Run } from "@/api/types"
import { StatusBadge } from "@/components/StatusBadge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { formatDateTime, formatDuration, shortId } from "@/lib/format"
import { failureText, runStatusStyle, triggerLabel } from "@/lib/status"
import { Link } from "react-router"

type Props = {
  runs: Run[]
  jobs: Map<string, Job>
  workers: Map<string, string>
}

const head = "h-auto px-5 py-3.5 text-[13px] font-medium text-subtle-foreground"
const cell = "h-[58px] px-5"
const link = "text-link hover:underline"

/** Runs as rows. Reused later on the job detail page. */
export function RunsTable({ runs, jobs, workers }: Props) {
  return (
    <Table>
      <TableHeader>
        <TableRow className="border-border hover:bg-transparent">
          <TableHead className={head}>Status</TableHead>
          <TableHead className={head}>Job</TableHead>
          <TableHead className={head}>Started</TableHead>
          <TableHead className={head}>Duration</TableHead>
          <TableHead className={head}>Trigger</TableHead>
          <TableHead className={head}>Worker</TableHead>
          <TableHead className={head}>Run</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {runs.map((run) => {
          const job = jobs.get(run.job_id)
          const failure = failureText(run)
          return (
            <TableRow key={run.id} className="border-border hover:bg-accent">
              <TableCell className={cell}>
                <div className="flex items-center gap-2">
                  <StatusBadge style={runStatusStyle[run.status]} />
                  {failure && <span className="text-xs text-subtle-foreground">{failure}</span>}
                </div>
              </TableCell>
              <TableCell className={cell}>
                <div className="flex flex-col leading-tight">
                  <span className="max-w-64 truncate">{job?.name ?? "Unknown job"}</span>
                  <span className="text-xs text-subtle-foreground">{job?.key ?? shortId(run.job_id)}</span>
                </div>
              </TableCell>
              <TableCell className={`${cell} tabular-nums text-subtle-foreground`}>
                {formatDateTime(run.started_at)}
              </TableCell>
              <TableCell className={`${cell} tabular-nums`}>
                {formatDuration(run.started_at, run.finished_at)}
              </TableCell>
              <TableCell className={cell}>{triggerLabel[run.trigger]}</TableCell>
              <TableCell className={`${cell} text-muted-foreground`}>
                {run.worker_id ? (workers.get(run.worker_id) ?? shortId(run.worker_id)) : "—"}
              </TableCell>
              <TableCell className={`${cell} tabular-nums`}>
                <Link to={`/runs/${run.id}`} className={link}>
                  {shortId(run.id)}
                </Link>
              </TableCell>
            </TableRow>
          )
        })}
      </TableBody>
    </Table>
  )
}
