import { useParams } from "react-router"

export function RunDetailPage() {
  const { id } = useParams()
  return <h1 className="text-[28px] font-semibold tracking-tight">Run {id}</h1>
}
