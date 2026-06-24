import type { User } from '../types'

export const AUTH_TOKEN_KEY = 'intelliread_access_token'
export const AUTH_USER_KEY = 'intelliread_user'

export function getStoredToken() {
  return localStorage.getItem(AUTH_TOKEN_KEY)
}

export function setStoredToken(token: string) {
  localStorage.setItem(AUTH_TOKEN_KEY, token)
}

export function clearStoredToken() {
  localStorage.removeItem(AUTH_TOKEN_KEY)
}

export function getStoredUser() {
  const raw = localStorage.getItem(AUTH_USER_KEY)
  return raw ? (JSON.parse(raw) as User) : null
}

export function setStoredUser(user: User) {
  localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user))
}

export function clearStoredUser() {
  localStorage.removeItem(AUTH_USER_KEY)
}
