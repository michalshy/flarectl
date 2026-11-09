import { useState } from "react"
import { Navigate, Route, Routes } from "react-router"
import { getToken } from "./api/client"
import { Layout } from "./components/Layout"
import { JobsPage } from "./pages/Jobs"
import { Login } from "./pages/Login"
import { RunDetailPage } from "./pages/RunDetail"
import { RunsPage } from "./pages/Runs"

function App() {
  const [loggedIn, setLoggedIn] = useState(getToken() !== null)

  if (!loggedIn) {
    return <Login onLogin={() => setLoggedIn(true)} />
  }

  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Navigate to="/runs" replace />} />
        <Route path="runs" element={<RunsPage />} />
        <Route path="runs/:id" element={<RunDetailPage />} />
        <Route path="jobs" element={<JobsPage />} />
        <Route path="*" element={<Navigate to="/runs" replace />} />
      </Route>
    </Routes>
  )
}

export default App
