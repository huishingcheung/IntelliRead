import type { ApiErrorResponse, ApiSuccessResponse } from '../types'
import { clearStoredToken, getStoredToken } from '../auth/authStorage'

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:3000/api/v1'

let onUnauthorizedHandler: (() => void) | null = null

export class ApiError extends Error {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.code = code
  }
}

export function registerUnauthorizedHandler(handler: (() => void) | null) {
  onUnauthorizedHandler = handler
}

type ApiOptions = RequestInit & {
  authenticated?: boolean
}

export async function api<T>(
  path: string,
  options: ApiOptions = {},
): Promise<T> {
  const headers = new Headers(options.headers)
  const token = getStoredToken()

  if (!(options.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  if (options.authenticated !== false && token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  let response: Response

  try {
    response = await fetch(`${API_BASE_URL}${path}`, {
      ...options,
      headers,
    })
  } catch {
    throw new ApiError('无法连接后端服务，请确认 http://127.0.0.1:3000 已启动。', 0)
  }

  if (response.status === 204) {
    return undefined as T
  }

  const text = await response.text()
  const payload = text ? JSON.parse(text) : null

  if (!response.ok) {
    const errorPayload = payload as ApiErrorResponse | null

    if (response.status === 401) {
      clearStoredToken()
      onUnauthorizedHandler?.()
    }

    throw new ApiError(
      errorPayload?.error.message ?? '请求失败',
      response.status,
      errorPayload?.error.code,
    )
  }

  return (payload as ApiSuccessResponse<T>).data
}

export { API_BASE_URL }
