import type { paths } from "./schema"
import createClient, { type Middleware } from "openapi-fetch"

const ADMIN_TOKEN = "flare_admin_token"

const authMiddleware: Middleware = {
  onRequest: ({ request }) => {
    const token = getToken()
    if (token) {
      request.headers.set("Authorization", `Bearer ${token}`)
    }
    return request
  },
  onResponse: ({ response }) => {
    if (response.status === 401) {
      clearToken()
    }
  },
}

export function getToken(): string | null {
  return localStorage.getItem(ADMIN_TOKEN)
}

export function saveToken(token: string): void {
  localStorage.setItem(ADMIN_TOKEN, token)
}

export function clearToken(): void {
  localStorage.removeItem(ADMIN_TOKEN)
}

export const client = createClient<paths>()
client.use(authMiddleware)
