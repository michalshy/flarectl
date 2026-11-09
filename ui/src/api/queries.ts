import { useInfiniteQuery, useQuery } from "@tanstack/react-query"
import { client } from "./client"
import type { RunsFilter } from "./types"

const RUNS_PAGE_SIZE = 50

/**
 * Runs, newest first, one page of 50 at a time. Each next page asks for runs that started
 * before the last run already loaded (`before=<started_at>`).
 * Refetched every 5 s so running runs update without a reload.
 * The filter is part of the query key: a different filter is a different cached query.
 */
export function useRuns(filter: Omit<RunsFilter, "before" | "limit"> = {}) {
  return useInfiniteQuery({
    queryKey: ["runs", filter],
    initialPageParam: undefined as string | undefined,
    queryFn: async ({ pageParam }) => {
      const { data, error } = await client.GET("/v1/runs", {
        params: { query: { ...filter, before: pageParam, limit: RUNS_PAGE_SIZE } },
      })
      if (error) throw error
      return data
    },
    // A short page means there's nothing older; undefined tells TanStack Query to stop.
    getNextPageParam: (lastPage) =>
      lastPage.length < RUNS_PAGE_SIZE ? undefined : lastPage[lastPage.length - 1].started_at,
    refetchInterval: 5_000,
  })
}

/** All registered workers. Used to show worker names next to runs, which only carry worker_id. */
export function useWorkers() {
  return useQuery({
    queryKey: ["workers"],
    queryFn: async () => {
      const { data, error } = await client.GET("/v1/workers")
      if (error) throw error
      return data
    },
  })
}

/** All jobs. Used to show job names next to runs, which only carry job_id. */
export function useJobs() {
  return useQuery({
    queryKey: ["jobs"],
    queryFn: async () => {
      const { data, error } = await client.GET("/v1/jobs")
      if (error) throw error
      return data
    },
  })
}
