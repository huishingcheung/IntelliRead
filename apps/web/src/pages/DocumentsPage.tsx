import { useCallback, useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { DocumentListItem, DocumentListResponse } from '../types'

function formatDate(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`
  return `${(value / (1024 * 1024)).toFixed(1)} MB`
}

export function DocumentsPage() {
  const [documents, setDocuments] = useState<DocumentListItem[]>([])
  const [query, setQuery] = useState('')
  const [showArchived, setShowArchived] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [busyId, setBusyId] = useState<string | null>(null)

  const fetchDocuments = useCallback(async () => {
    setIsLoading(true)
    setError('')

    try {
      const response = await api<DocumentListResponse>(
        `/documents?limit=50&offset=0&archived=${showArchived}${query ? `&q=${encodeURIComponent(query)}` : ''}`,
      )
      setDocuments(response.items)
    } catch (fetchError) {
      setError(fetchError instanceof ApiError ? fetchError.message : '加载文献列表失败。')
    } finally {
      setIsLoading(false)
    }
  }, [query, showArchived])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void fetchDocuments()
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [fetchDocuments])

  const totals = useMemo(
    () => ({
      total: documents.length,
      markdown: documents.filter((item) => item.source_type === 'markdown').length,
      txt: documents.filter((item) => item.source_type === 'txt').length,
    }),
    [documents],
  )

  const handleArchiveToggle = async (document: DocumentListItem) => {
    setBusyId(document.id)
    setError('')

    try {
      await api(`/documents/${document.id}`, {
        method: 'PATCH',
        body: JSON.stringify({ archived: !document.archived_at }),
      })
      await fetchDocuments()
    } catch (actionError) {
      setError(actionError instanceof ApiError ? actionError.message : '更新文献状态失败。')
    } finally {
      setBusyId(null)
    }
  }

  const handleDelete = async (document: DocumentListItem) => {
    const confirmed = window.confirm(`确认删除《${document.title}》吗？删除后无法恢复。`)
    if (!confirmed) return

    setBusyId(document.id)
    setError('')

    try {
      await api(`/documents/${document.id}`, {
        method: 'DELETE',
      })
      await fetchDocuments()
    } catch (actionError) {
      setError(actionError instanceof ApiError ? actionError.message : '删除文献失败。')
    } finally {
      setBusyId(null)
    }
  }

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] px-6 py-8">
      <div className="mx-auto max-w-6xl">
        <div className="mb-6 flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">文献管理</p>
            <h1 className="mt-2 font-[var(--font-reading)] text-4xl text-[var(--ink-strong)]">
              文献库
            </h1>
          </div>
          <Link
            className="rounded-full border border-[var(--line-strong)] px-5 py-2.5 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]"
            to="/"
          >
            返回首页
          </Link>
        </div>

        <div className="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
          <aside className="rounded-[24px] border border-[var(--line-muted)] bg-[rgba(255,255,255,0.42)] p-6 shadow-[var(--shadow-soft)]">
            <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">筛选与概览</p>
            <div className="mt-6 space-y-4">
              <div className="rounded-[20px] border border-[var(--line-muted)] bg-white/60 p-4">
                <div className="text-sm text-[var(--ink-soft)]">当前列表</div>
                <div className="mt-2 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">
                  {totals.total} 篇
                </div>
              </div>
              <div className="rounded-[20px] border border-[var(--line-muted)] bg-white/60 p-4">
                <div className="text-sm text-[var(--ink-soft)]">Markdown</div>
                <div className="mt-2 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">
                  {totals.markdown}
                </div>
              </div>
              <div className="rounded-[20px] border border-[var(--line-muted)] bg-white/60 p-4">
                <div className="text-sm text-[var(--ink-soft)]">TXT</div>
                <div className="mt-2 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">
                  {totals.txt}
                </div>
              </div>
            </div>
          </aside>

          <section className="rounded-[24px] border border-[var(--line-muted)] bg-[rgba(255,255,255,0.42)] p-6 shadow-[var(--shadow-soft)]">
            <div className="flex flex-wrap items-center gap-4">
              <input
                type="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="搜索标题或正文内容"
                className="min-w-[240px] flex-1 rounded-[18px] border border-[var(--line-muted)] bg-white/70 px-4 py-3 outline-none"
              />
              <button
                onClick={() => setShowArchived(false)}
                className={`rounded-full px-4 py-2 text-sm transition ${
                  !showArchived
                    ? 'bg-[var(--ink-strong)] text-white'
                    : 'border border-[var(--line-strong)] text-[var(--ink-main)]'
                }`}
              >
                进行中
              </button>
              <button
                onClick={() => setShowArchived(true)}
                className={`rounded-full px-4 py-2 text-sm transition ${
                  showArchived
                    ? 'bg-[var(--ink-strong)] text-white'
                    : 'border border-[var(--line-strong)] text-[var(--ink-main)]'
                }`}
              >
                已归档
              </button>
            </div>

            <div className="mt-6">
              {isLoading ? (
                <div className="py-10 text-center text-[var(--ink-soft)]">正在加载文献列表...</div>
              ) : error ? (
                <div className="rounded-[18px] border border-[rgba(180,90,90,0.22)] bg-[rgba(255,240,240,0.7)] px-4 py-3 text-sm text-[#8b3d3d]">
                  {error}
                </div>
              ) : documents.length === 0 ? (
                <div className="py-10 text-center text-[var(--ink-soft)]">
                  这里还没有文献，先回首页导入一篇吧。
                </div>
              ) : (
                <div className="space-y-4">
                  {documents.map((document) => {
                    const isBusy = busyId === document.id

                    return (
                      <div
                        key={document.id}
                        className="rounded-[20px] border border-[var(--line-muted)] bg-white/55 px-5 py-4"
                      >
                        <div className="flex flex-wrap items-start justify-between gap-4">
                          <Link to={`/documents/${document.id}`} className="min-w-0 flex-1">
                            <p className="font-[var(--font-reading)] text-xl text-[var(--ink-strong)]">
                              {document.title}
                            </p>
                            <p className="mt-2 text-sm text-[var(--ink-soft)]">
                              {document.original_filename} · {formatBytes(document.byte_size)} · 最近更新 {formatDate(document.updated_at)}
                            </p>
                            <p className="mt-1 text-sm text-[var(--ink-soft)]">
                              类型：{document.source_type === 'markdown' ? 'Markdown' : 'TXT'}
                            </p>
                          </Link>

                          <div className="flex flex-wrap items-center gap-2">
                            <span className="rounded-full bg-[rgba(50,116,109,0.12)] px-3 py-1 text-xs text-[var(--ink-strong)]">
                              {document.archived_at ? '已归档' : '进行中'}
                            </span>
                            <Link
                              to={`/documents/${document.id}/notes-tags`}
                              className="rounded-full border border-[var(--line-strong)] px-4 py-2 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]"
                            >
                              标签与笔记
                            </Link>
                            <button
                              disabled={isBusy}
                              onClick={() => handleArchiveToggle(document)}
                              className="rounded-full border border-[var(--line-strong)] px-4 py-2 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:opacity-60"
                            >
                              {document.archived_at ? '恢复' : '归档'}
                            </button>
                            <button
                              disabled={isBusy}
                              onClick={() => handleDelete(document)}
                              className="rounded-full border border-[rgba(180,90,90,0.35)] px-4 py-2 text-sm text-[#8b3d3d] transition hover:bg-[rgba(255,240,240,0.7)] disabled:opacity-60"
                            >
                              删除
                            </button>
                          </div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}
