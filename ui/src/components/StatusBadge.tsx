import { Badge } from "@/components/ui/badge"
import type { StatusStyle } from "@/lib/status"
import { cn } from "cn"

/** Status pill: a filled or hollow dot, then the label. Pass runStatusStyle[x] or stepStatusStyle[x]. */
export function StatusBadge({ style }: { style: StatusStyle }) {
  return (
    <Badge className={cn("h-auto gap-1.5 rounded-lg px-2.5 py-0.75 text-xs font-medium", style.badge)}>
      <span
        aria-hidden
        className={cn("size-2 rounded-full border-2 border-current", style.filled && "bg-current")}
      />
      {style.label}
    </Badge>
  )
}
