import { useState, type FormEvent } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { ApiError } from '../api/client'
import { useAuth } from '../auth/useAuth'

const submitStyle = {
  width: '100%',
  border: 'none',
  borderRadius: '9999px',
  backgroundColor: '#163b36',
  color: '#ffffff',
  padding: '16px 20px',
  fontSize: '16px',
  fontWeight: 600,
} as const

export function RegisterPage() {
  const navigate = useNavigate()
  const { register } = useAuth()
  const [username, setUsername] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setError('')
    setIsSubmitting(true)

    try {
      await register(username, email, password)
      navigate('/', { replace: true })
    } catch (submitError) {
      if (submitError instanceof ApiError) {
        setError(submitError.message)
      } else {
        setError('注册失败，请稍后再试。')
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] px-6 py-8">
      <div className="mx-auto max-w-2xl">
        <div className="rounded-[28px] border border-[var(--line-muted)] bg-[rgba(255,255,255,0.56)] p-8 shadow-[var(--shadow-soft)] sm:p-10">
          <p className="font-[var(--font-ui)] text-sm tracking-[0.18em] text-[var(--ink-soft)]">IntelliRead</p>
          <h1 className="mt-4 font-[var(--font-reading)] text-5xl text-[var(--ink-strong)]">注册</h1>
          <p className="mt-4 text-sm leading-8 text-[var(--ink-soft)]">
            注册成功后会自动登录，你可以直接进入文献导入和阅读流程。
          </p>

          <form className="mt-8" onSubmit={handleSubmit}>
            <div className="space-y-5">
              <label className="block">
                <span className="mb-3 block text-sm text-[var(--ink-strong)]">用户名</span>
                <input
                  type="text"
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                  className="w-full rounded-[18px] border border-[var(--line-muted)] bg-white/85 px-5 py-4 text-lg outline-none"
                  placeholder="student01"
                  required
                />
              </label>

              <label className="block">
                <span className="mb-3 block text-sm text-[var(--ink-strong)]">邮箱</span>
                <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  className="w-full rounded-[18px] border border-[var(--line-muted)] bg-white/85 px-5 py-4 text-lg outline-none"
                  placeholder="student01@example.com"
                  required
                />
              </label>

              <label className="block">
                <span className="mb-3 block text-sm text-[var(--ink-strong)]">密码</span>
                <input
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  className="w-full rounded-[18px] border border-[var(--line-muted)] bg-white/85 px-5 py-4 text-lg outline-none"
                  placeholder="至少 8 位"
                  required
                />
              </label>
            </div>

            {error ? (
              <div className="mt-5 rounded-[18px] border border-[rgba(180,90,90,0.22)] bg-[rgba(255,240,240,0.7)] px-4 py-3 text-sm text-[#8b3d3d]">
                {error}
              </div>
            ) : null}

            <div className="mt-6">
              <button type="submit" disabled={isSubmitting} style={submitStyle}>
                {isSubmitting ? '注册中...' : '立即注册'}
              </button>
            </div>
          </form>

          <div className="mt-8 flex items-center justify-between gap-3 text-sm text-[var(--ink-soft)]">
            <span>已有账号？</span>
            <Link className="text-[var(--accent)]" to="/login">
              去登录
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}
