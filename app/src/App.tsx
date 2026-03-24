import { ErrorBoundary } from './components/ErrorBoundary'
import { ExomApp } from './components/exom/exom-app'
import { TooltipProvider } from './components/ui/tooltip'

export function App() {
  return (
    <ErrorBoundary>
      <TooltipProvider delayDuration={100}>
        <ExomApp />
      </TooltipProvider>
    </ErrorBoundary>
  )
}
