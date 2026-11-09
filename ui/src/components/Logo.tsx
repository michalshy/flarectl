/**
 * Flare mark: a spark rising over a horizon line, a signal sent up so someone notices.
 * The spark is drawn in currentColor, so the caller sets its colour (text-primary) and size (size-6);
 * the horizon uses the dark moss logo token.
 */
export function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 32 32" aria-hidden className={className}>
      <path
        fill="currentColor"
        d="M16 2Q19 11.38 28 14.5Q21.66 16.7 18.3 22H13.7Q10.34 16.7 4 14.5Q13 11.38 16 2Z"
      />
      <path
        className="fill-logo-horizon"
        d="M3 25.5H29A1 1 0 0 1 30 26.5V28.5A1 1 0 0 1 29 29.5H3A1 1 0 0 1 2 28.5V26.5A1 1 0 0 1 3 25.5Z"
      />
    </svg>
  )
}
