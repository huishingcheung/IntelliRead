import {
  useEffect,
  useState,
  useCallback,
  type ReactNode,
} from 'react'
import {
  api,
  registerUnauthorizedHandler,
} from '../api/client'
import type { AuthPayload, User } from '../types'
import {
  clearStoredToken,
  clearStoredUser,
  getStoredToken,
  getStoredUser,
  setStoredToken,
  setStoredUser,
} from './authStorage'
import { AuthContext, type AuthContextValue } from './authContext'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => getStoredToken())
  const [user, setUser] = useState<User | null>(() => getStoredUser())
  const [isBootstrapping] = useState(false)

  useEffect(() => {
    registerUnauthorizedHandler(() => {
      setToken(null)
      setUser(null)
      clearStoredToken()
      clearStoredUser()
    })

    return () => {
      registerUnauthorizedHandler(null)
    }
  }, [])

  const persistAuth = useCallback((payload: AuthPayload) => {
    setStoredToken(payload.access_token)
    setStoredUser(payload.user)
    setToken(payload.access_token)
    setUser(payload.user)
  }, [])

  const login = useCallback(async (email: string, password: string) => {
    const payload = await api<AuthPayload>('/auth/login', {
      method: 'POST',
      authenticated: false,
      body: JSON.stringify({ email, password }),
    })

    persistAuth(payload)
  }, [persistAuth])

  const register = useCallback(async (username: string, email: string, password: string) => {
    await api<User>('/auth/register', {
      method: 'POST',
      authenticated: false,
      body: JSON.stringify({ username, email, password }),
    })

    await login(email, password)
  }, [login])

  const logout = useCallback(() => {
    clearStoredToken()
    clearStoredUser()
    setToken(null)
    setUser(null)
  }, [])

  const value: AuthContextValue = {
    user,
    token,
    isAuthenticated: Boolean(token && user),
    isBootstrapping,
    login,
    register,
    logout,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}
