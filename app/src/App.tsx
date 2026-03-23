import { ErrorBoundary } from './components/ErrorBoundary'
import { ExomApp } from './components/exom/exom-app'

export function App() {
  return (
    <ErrorBoundary>
      <ExomApp />
    </ErrorBoundary>
  )
}
