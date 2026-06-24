import { useCallback, useEffect, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import type { DocumentDetail, Note, Tag } from '../types'

function formatDate(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

export function DocumentNotesTagsPage() {
  const { id = '' } = useParams()

  const [document, setDocument] = useState<DocumentDetail | null>(null)
  const [tags, setTags] = useState<Tag[]>([])
  const [notes, setNotes] = useState<Note[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')

  const fetchPageData = useCallback(async () => {
    setIsLoading(true)
    setError('')

    try {
      const [documentData, tagData, noteData] = await Promise.all([
        api<DocumentDetail>(`/documents/${id}`),
        api<Tag[]>(`/documents/${id}/tags`),
        api<Note[]>(`/documents/${id}/notes`),
      ])

      setDocument(documentData)
      setTags(tagData)
      setNotes(noteData)
    } catch (fetchError) {
      setError(fetchError instanceof ApiError ? fetchError.message : '加载标签与笔记失败。')
    } finally {
      setIsLoading(false)
    }
  }, [id])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void fetchPageData()
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [fetchPageData])

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] px-6 py-8">
      <div className="mx-auto max-w-6xl">
        <header className="mb-6 flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">标签与笔记</p>
            <h1 className="mt-2 font-[var(--font-reading)] text-4xl text-[var(--ink-strong)]">
              {document?.title ?? '文献标签与笔记'}
            </h1>
            {document ? (
              <p className="mt-2 text-sm text-[var(--ink-soft)]">
                {document.original_filename} · 共 {document.paragraphs.length} 段
              </p>
            ) : null}
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <Link
              className="rounded-full border border-[var(--line-strong)] px-5 py-2.5 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]"
              to="/documents"
            >
              返回文献库
            </Link>
            {id ? (
              <Link
                className="rounded-full border border-[var(--line-strong)] px-5 py-2.5 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]"
                to={`/documents/${id}`}
              >
                返回阅读页
              </Link>
            ) : null}
          </div>
        </header>

        {isLoading ? (
          <div className="rounded-[24px] border border-[var(--line-muted)] bg-white/50 px-6 py-10 text-center text-[var(--ink-soft)]">
            正在加载标签与笔记...
          </div>
        ) : error ? (
          <div className="rounded-[18px] border border-[rgba(180,90,90,0.22)] bg-[rgba(255,240,240,0.7)] px-4 py-3 text-sm text-[#8b3d3d]">
            {error}
          </div>
        ) : !document ? (
          <div className="rounded-[24px] border border-[var(--line-muted)] bg-white/50 px-6 py-10 text-center text-[var(--ink-soft)]">
            文献不存在。
          </div>
        ) : (
          <div className="grid gap-6 lg:grid-cols-[300px_minmax(0,1fr)]">
            <aside className="rounded-[24px] border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-6 shadow-[var(--shadow-soft)]">
              <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">文献标签</p>
              <div className="mt-5 flex flex-wrap gap-3">
                {tags.length === 0 ? (
                  <p className="text-sm leading-7 text-[var(--ink-soft)]">这一篇文献还没有标签。</p>
                ) : (
                  tags.map((tag) => (
                    <span
                      key={tag.id}
                      className="rounded-full bg-[rgba(50,116,109,0.12)] px-4 py-2 text-sm text-[var(--ink-strong)]"
                    >
                      {tag.name}
                    </span>
                  ))
                )}
              </div>
            </aside>

            <section className="rounded-[24px] border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-6 shadow-[var(--shadow-soft)]">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">阅读笔记</p>
                  <h2 className="mt-2 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">
                    这一篇的全部笔记
                  </h2>
                </div>
                <div className="rounded-full bg-[rgba(50,116,109,0.12)] px-4 py-2 text-sm text-[var(--ink-strong)]">
                  共 {notes.length} 条
                </div>
              </div>

              <div className="mt-6 space-y-4">
                {notes.length === 0 ? (
                  <div className="rounded-[20px] border border-[var(--line-muted)] bg-white/60 px-5 py-5 text-sm text-[var(--ink-soft)]">
                    这一篇文献还没有笔记。
                  </div>
                ) : (
                  notes.map((note) => {
                    const paragraph = note.paragraph_id
                      ? document.paragraphs.find((item) => item.id === note.paragraph_id)
                      : null

                    return (
                      <article
                        key={note.id}
                        className="rounded-[20px] border border-[var(--line-muted)] bg-white/60 px-5 py-4"
                      >
                        <div className="flex flex-wrap items-center justify-between gap-3">
                          <div className="text-sm text-[var(--ink-soft)]">
                            {paragraph ? `第 ${paragraph.position + 1} 段笔记` : '整篇文献笔记'}
                          </div>
                          <div className="text-xs text-[var(--ink-soft)]">{formatDate(note.updated_at)}</div>
                        </div>
                        <p className="mt-3 text-sm leading-7 text-[var(--ink-main)]">{note.content}</p>
                      </article>
                    )
                  })
                )}
              </div>
            </section>
          </div>
        )}
      </div>
    </div>
  )
}
