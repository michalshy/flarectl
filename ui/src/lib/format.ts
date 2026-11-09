const dateTime = new Intl.DateTimeFormat(undefined, {
  month: "short",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
})

/** "Nov 9, 14:32:05" in the browser's locale and time zone. */
export function formatDateTime(iso: string): string {
  return dateTime.format(new Date(iso))
}

/**
 * Time between two timestamps: "850ms", "12s", "3m 05s", "1h 02m".
 * `end` is null while the run is still going; then the duration counts up to `now`.
 */
export function formatDuration(start: string, end: string | null, now = Date.now()): string {
  const ms = Math.max(0, (end ? Date.parse(end) : now) - Date.parse(start))
  if (ms < 1000) return `${ms}ms`

  const totalSeconds = Math.floor(ms / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  const pad = (n: number) => String(n).padStart(2, "0")

  if (hours > 0) return `${hours}h ${pad(minutes)}m`
  if (minutes > 0) return `${minutes}m ${pad(seconds)}s`
  return `${seconds}s`
}

/** First 8 characters of a uuid, enough to tell runs apart on screen. */
export function shortId(id: string): string {
  return id.slice(0, 8)
}
