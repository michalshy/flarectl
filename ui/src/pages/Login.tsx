import { clearToken, client, saveToken } from "@/api/client"
import { Logo } from "@/components/Logo"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { useState } from "react"

export function Login({ onLogin }: { onLogin: () => void }) {
  const [token, setToken] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  async function handleSubmit(e: React.SubmitEvent<HTMLFormElement>) {
    e.preventDefault()
    setError(null)
    setSubmitting(true)
    saveToken(token)
    try {
      const { error: apiError, response } = await client.GET("/v1/jobs")
      if (apiError) {
        clearToken()
        setError(response.status === 401 ? "Invalid token" : apiError.error)
      } else {
        onLogin()
      }
    } catch {
      clearToken()
      setError("Can't reach the server")
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-svh items-center justify-center px-4 py-10">
      <div className="glass flex w-full max-w-sm flex-col gap-8 rounded-[18px] p-8">
        <div className="flex flex-col items-center gap-3 text-center">
          <Logo className="size-14 text-primary" />
          <div className="flex flex-col gap-1">
            <h1 className="text-[28px] font-semibold tracking-tight">Flare</h1>
            <p className="text-sm text-muted-foreground">Runs, steps and logs of your jobs</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label htmlFor="admin-token" className="text-[13px] font-medium text-subtle-foreground">
              Admin token
            </label>
            <Input
              id="admin-token"
              type="password"
              autoComplete="current-password"
              autoFocus
              value={token}
              onChange={(e) => setToken(e.target.value)}
              aria-invalid={error !== null}
              aria-describedby={error ? "admin-token-error" : undefined}
            />
            {error && (
              <p id="admin-token-error" role="alert" className="text-[13px] text-destructive">
                {error}
              </p>
            )}
          </div>
          <Button type="submit" className="w-full" disabled={submitting || token === ""}>
            {submitting ? "Signing in…" : "Sign in"}
          </Button>
        </form>

        <p className="text-center text-xs text-subtle-foreground">
          The token is set with <span className="font-medium">FLARE_ADMIN_TOKEN</span> on the server.
        </p>
      </div>
    </main>
  )
}
