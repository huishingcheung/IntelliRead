import { createContext } from 'react'
import type { User } from '../types'

export type AuthContextValue = {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isBootstrapping: boolean
  login: (email: string, password: string) => Promise<void>
  register: (username: string, email: string, password: string) => Promise<void>
  logout: () => void
}

export const AuthContext = createContext<AuthContextValue | null>(null)
