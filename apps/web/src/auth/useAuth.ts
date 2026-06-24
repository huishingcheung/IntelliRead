import { useContext } from 'react'
import { AuthContext } from './authContext'

export function useAuth() {
  const context = useContext(AuthContext)

  if (!context) {
    throw new Error('useAuth 必须在 AuthProvider 中使用')
  }

  return context
}
