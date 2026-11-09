/**
 * Soft colour blobs behind the whole app. Glass panels blur what's behind them,
 * so without these the glass would look flat. Purely decorative and static.
 */
export function Backdrop() {
  return (
    <div aria-hidden className="pointer-events-none fixed inset-0 -z-10 overflow-hidden">
      <div className="absolute -top-40 -left-32 size-140 rounded-full bg-blob-green opacity-50 blur-[120px]" />
      <div className="absolute -top-24 right-[-10%] size-115 rounded-full bg-blob-yellow opacity-30 blur-[120px]" />
      <div className="absolute -bottom-48 left-[20%] size-155 rounded-full bg-blob-brown opacity-50 blur-[120px]" />
      <div className="absolute right-[-8%] -bottom-32 size-105 rounded-full bg-blob-khaki opacity-30 blur-[120px]" />
    </div>
  )
}
