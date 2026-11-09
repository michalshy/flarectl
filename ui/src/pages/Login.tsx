import { clearToken, client, saveToken } from "@/api/client"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { useState } from "react"

export function Login({ onLogin }: { onLogin: () => void }) {
  const [token, setToken] = useState("")
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(e: React.SubmitEvent<HTMLFormElement>) {
    e.preventDefault()
    setError(null)
    saveToken(token)
    try {
      const { error } = await client.GET("/v1/jobs")
      if (error) {
        clearToken()
        setError(error.error)
      }
      else {
        onLogin()
      }
    }
    catch {
      clearToken();
      setError("Can't reach the server")
    }
  }

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <Input value={token} onChange={(e) => setToken(e.target.value)} placeholder="admin token" type="password"/>
        <Button type="submit">login</Button>
      </form>
      {error && <p>{error}</p>}
    </div>
  )
}
