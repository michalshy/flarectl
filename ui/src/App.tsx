import { useState } from "react"
import { getToken } from "./api/client"
import { Login } from "./pages/Login"

function App() {
  const [loggedIn, setLoggedIn] = useState(getToken() !== null)

  if (!loggedIn) {
    return <Login onLogin={() => setLoggedIn(true)} />
  }
  return <h1>Flare</h1>
}

export default App
