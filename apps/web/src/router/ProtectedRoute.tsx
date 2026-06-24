import { Navigate, Outlet, useLocation } from 'react-router-dom'
import { useAuth } from '../auth/useAuth'

export function ProtectedRoute() {
  const { isAuthenticated, isBootstrapping } = useAuth()
  const location = useLocation()

  if (isBootstrapping) {
    return (
      <div className="flex min-h-screen items-center justify-center text-[var(--ink-soft)]">
        正在恢复登录状态...
      </div>
    )
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace state={{ from: location }} />
  }

  return <Outlet />
}
