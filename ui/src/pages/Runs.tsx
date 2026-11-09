import { useJobs, useRuns, useWorkers } from "@/api/queries"
import { RunsFilters } from "@/components/RunsFilters"
import { RunsTable } from "@/components/RunsTable"
import { Button } from "@/components/ui/button"
import { isRunStatus } from "@/lib/status"
import { useSearchParams } from "react-router"

export function RunsPage() {
  // Filters live in the URL (/runs?status=failed&job=<id>): they survive a refresh and can be shared.
  const [params, setParams] = useSearchParams()
  const statusParam = params.get("status")
  const status = isRunStatus(statusParam) ? statusParam : undefined
  const jobId = params.get("job") ?? undefined

  function setParam(key: string, value: string | undefined) {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev)
        if (value) next.set(key, value)
        else next.delete(key)
        return next
      },
      { replace: true }, // changing a filter shouldn't add a Back-button step
    )
  }

  const runs = useRuns({ status, job_id: jobId })
  const jobs = useJobs()
  const workers = useWorkers()

  // Lookups by id, so each row finds its job and worker without searching the whole list.
  const jobsById = new Map((jobs.data ?? []).map((job) => [job.id, job]))
  const workersById = new Map((workers.data ?? []).map((w) => [w.id, w.name]))
  const filtered = status !== undefined || jobId !== undefined
  // Pages joined into one list for the table.
  const allRuns = runs.data?.pages.flat() ?? []

  return (
    <div className="flex flex-col gap-6">
      <h1 className="text-[28px] font-semibold tracking-tight">Runs</h1>

      <RunsFilters
        status={status}
        jobId={jobId}
        jobs={jobs.data ?? []}
        onStatusChange={(s) => setParam("status", s)}
        onJobChange={(id) => setParam("job", id)}
      />

      <div className="glass overflow-hidden rounded-[18px]">
        {runs.isPending ? (
          <Message>Loading runs…</Message>
        ) : runs.isError ? (
          <Message>Couldn't load runs.</Message>
        ) : allRuns.length === 0 ? (
          <Message>{filtered ? "No runs match these filters." : "No runs yet."}</Message>
        ) : (
          <RunsTable runs={allRuns} jobs={jobsById} workers={workersById} />
        )}
      </div>

      {runs.hasNextPage && (
        <Button
          variant="secondary"
          className="self-center"
          onClick={() => runs.fetchNextPage()}
          disabled={runs.isFetchingNextPage}
        >
          {runs.isFetchingNextPage ? "Loading…" : "Load more"}
        </Button>
      )}
    </div>
  )
}

function Message({ children }: { children: React.ReactNode }) {
  return <p className="px-5 py-10 text-center text-muted-foreground">{children}</p>
}
