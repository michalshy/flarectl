import type { Job, RunStatus } from "@/api/types"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { runStatuses, runStatusStyle } from "@/lib/status"

/** Stands for "no filter" in the controls; never sent to the API. */
const ALL = "all"

type Props = {
  status: RunStatus | undefined
  jobId: string | undefined
  jobs: Job[]
  onStatusChange: (status: RunStatus | undefined) => void
  onJobChange: (jobId: string | undefined) => void
}

/** Status tabs and a job select. Controlled: the parent owns the values (kept in the URL). */
export function RunsFilters({ status, jobId, jobs, onStatusChange, onJobChange }: Props) {
  // Value -> label pairs; the Select uses them to show the chosen job's name in the trigger.
  const jobItems = [
    { value: ALL, label: "All jobs" },
    ...jobs.map((job) => ({ value: job.id, label: job.name })),
  ]

  return (
    <div className="flex flex-wrap items-center gap-3">
      <Tabs
        value={status ?? ALL}
        onValueChange={(value: RunStatus | typeof ALL) => onStatusChange(value === ALL ? undefined : value)}
      >
        <TabsList>
          <TabsTrigger value={ALL} className="h-[34px]">
            All
          </TabsTrigger>
          {runStatuses.map((s) => (
            <TabsTrigger key={s} value={s} className="h-[34px]">
              {runStatusStyle[s].label}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>

      <Select
        items={jobItems}
        value={jobId ?? ALL}
        onValueChange={(value) => onJobChange(value && value !== ALL ? value : undefined)}
      >
        <SelectTrigger className="min-w-52">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {jobItems.map((item) => (
            <SelectItem key={item.value} value={item.value}>
              {item.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
