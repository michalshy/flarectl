import './index.css'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router'
import App from './App.tsx'
import { Backdrop } from './components/Backdrop'

// One cache for the whole app: pages that ask for the same data share it.
const queryClient = new QueryClient()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Backdrop />
        <App />
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
)
