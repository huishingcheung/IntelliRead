import { useCallback, useEffect, useMemo, useState, type FormEvent } from 'react'
import { Link } from 'react-router-dom'
import { ApiError } from '../api/client'
import {
  deleteVocabulary,
  listVocabulary,
  updateVocabulary,
} from '../api/vocabulary'
import { useAuth } from '../auth/useAuth'
import type { MasteryStatus, VocabularyCard } from '../types'

const statusOptions: Array<{ value: MasteryStatus | 'all'; label: string }> = [
  { value: 'all', label: '全部状态' },
  { value: 'new', label: '新词' },
  { value: 'learning', label: '学习中' },
  { value: 'familiar', label: '较熟悉' },
  { value: 'mastered', label: '已掌握' },
]

const statusLabels: Record<MasteryStatus, string> = {
  new: '新词',
  learning: '学习中',
  familiar: '较熟悉',
  mastered: '已掌握',
}

const statusClasses: Record<MasteryStatus, string> = {
  new: 'border-[#a16d2d] bg-[#f8efe2] text-[#80551f]',
  learning: 'border-[#32746d] bg-[#e3efeb] text-[#245c57]',
  familiar: 'border-[#58769a] bg-[#e9eff6] text-[#405f83]',
  mastered: 'border-[#557b48] bg-[#e7f0e3] text-[#426438]',
}

const PAGE_SIZE = 20

function formatDate(value: string | null) {
  if (!value) return '尚未安排'

  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

export function VocabularyPage() {
  const { user, logout } = useAuth()
  const [cards, setCards] = useState<VocabularyCard[]>([])
  const [page, setPage] = useState(1)
  const [total, setTotal] = useState(0)
  const [masteryStatus, setMasteryStatus] = useState<MasteryStatus | 'all'>('all')
  const [sort, setSort] = useState<'created_at' | 'next_review_at' | 'term'>('created_at')
  const [order, setOrder] = useState<'asc' | 'desc'>('desc')
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [busyId, setBusyId] = useState<string | null>(null)
  const [editingCard, setEditingCard] = useState<VocabularyCard | null>(null)
  const [definition, setDefinition] = useState('')
  const [exampleSentence, setExampleSentence] = useState('')
  const [editStatus, setEditStatus] = useState<MasteryStatus>('new')

  const fetchCards = useCallback(async () => {
    setIsLoading(true)
    setError('')

    try {
      const response = await listVocabulary({
        page,
        pageSize: PAGE_SIZE,
        sort,
        order,
        masteryStatus,
      })
      setCards(response.items)
      setTotal(response.total)
    } catch (fetchError) {
      setError(fetchError instanceof ApiError ? fetchError.message : '加载生词本失败。')
    } finally {
      setIsLoading(false)
    }
  }, [masteryStatus, order, page, sort])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void fetchCards()
    }, 0)

    return () => window.clearTimeout(timer)
  }, [fetchCards])

  const totals = useMemo(
    () => ({
      current: cards.length,
      newWords: cards.filter((card) => card.mastery_status === 'new').length,
    }),
    [cards],
  )

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))

  const startEditing = (card: VocabularyCard) => {
    setEditingCard(card)
    setDefinition(card.definition)
    setExampleSentence(card.example_sentence ?? '')
    setEditStatus(card.mastery_status)
    setError('')
  }

  const handleUpdate = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!editingCard || !definition.trim()) return

    setBusyId(editingCard.id)
    setError('')

    try {
      await updateVocabulary(editingCard.id, {
        definition: definition.trim(),
        example_sentence: exampleSentence.trim(),
        mastery_status: editStatus,
      })
      setEditingCard(null)
      await fetchCards()
    } catch (updateError) {
      setError(updateError instanceof ApiError ? updateError.message : '更新生词失败。')
    } finally {
      setBusyId(null)
    }
  }

  const handleDelete = async (card: VocabularyCard) => {
    if (!window.confirm(`确认从生词本删除“${card.term}”吗？`)) return

    setBusyId(card.id)
    setError('')

    try {
      await deleteVocabulary(card.id)
      if (cards.length === 1 && page > 1) {
        setPage((current) => current - 1)
      } else {
        await fetchCards()
      }
    } catch (deleteError) {
      setError(deleteError instanceof ApiError ? deleteError.message : '删除生词失败。')
    } finally {
      setBusyId(null)
    }
  }

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] text-[var(--ink-main)]">
      <header className="border-b border-[var(--line-muted)] bg-[rgba(244,248,244,0.82)] px-6 py-4">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-4">
          <nav className="flex flex-wrap items-center gap-6 text-sm text-[var(--ink-strong)]">
            <Link className="font-[var(--font-display)] text-xl" to="/">IntelliRead</Link>
            <Link className="transition hover:text-[var(--accent)]" to="/documents">文献库</Link>
            <Link className="font-medium text-[var(--accent)]" to="/vocabulary">生词本</Link>
            <Link className="transition hover:text-[var(--accent)]" to="/review">复习</Link>
          </nav>
          <div className="flex items-center gap-3 text-sm">
            <span className="text-[var(--ink-soft)]">{user?.username}</span>
            <button onClick={logout} className="border border-[var(--line-strong)] px-3 py-1.5 transition hover:border-[var(--accent)] hover:text-[var(--accent)]">退出</button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-6 py-8">
        <div className="flex flex-wrap items-end justify-between gap-5 border-b border-[var(--line-strong)] pb-5">
          <div>
            <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">VOCABULARY</p>
            <h1 className="mt-2 font-[var(--font-reading)] text-3xl text-[var(--ink-strong)]">生词本</h1>
          </div>
          <Link to="/review" className="border border-[#104f55] bg-[#104f55] px-5 py-2.5 text-sm text-white transition hover:bg-[#0d4348]">开始复习</Link>
        </div>

        <section className="grid gap-4 border-b border-[var(--line-muted)] py-5 md:grid-cols-[minmax(0,1fr)_180px_180px_180px]">
          <div className="flex items-center gap-6 text-sm text-[var(--ink-soft)]">
            <span>共 {total} 条</span>
            <span>本页 {totals.current} 条</span>
            <span>本页新词 {totals.newWords} 条</span>
          </div>
          <select aria-label="掌握状态" value={masteryStatus} onChange={(event) => { setMasteryStatus(event.target.value as MasteryStatus | 'all'); setPage(1) }} className="border border-[var(--line-muted)] bg-white/70 px-3 py-2 text-sm outline-none">
            {statusOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
          </select>
          <select aria-label="排序字段" value={sort} onChange={(event) => { setSort(event.target.value as typeof sort); setPage(1) }} className="border border-[var(--line-muted)] bg-white/70 px-3 py-2 text-sm outline-none">
            <option value="created_at">按加入时间</option>
            <option value="next_review_at">按复习时间</option>
            <option value="term">按词汇名称</option>
          </select>
          <select aria-label="排序方向" value={order} onChange={(event) => { setOrder(event.target.value as typeof order); setPage(1) }} className="border border-[var(--line-muted)] bg-white/70 px-3 py-2 text-sm outline-none">
            <option value="desc">降序</option>
            <option value="asc">升序</option>
          </select>
        </section>

        {error ? <div className="mt-5 border border-[rgba(180,90,90,0.3)] bg-[#fff0f0] px-4 py-3 text-sm text-[#8b3d3d]">{error}</div> : null}

        <section className="mt-6">
          {isLoading ? (
            <div className="py-16 text-center text-[var(--ink-soft)]">正在加载生词...</div>
          ) : cards.length === 0 ? (
            <div className="border border-dashed border-[var(--line-strong)] px-6 py-14 text-center">
              <p className="font-[var(--font-reading)] text-xl text-[var(--ink-strong)]">生词本还是空的</p>
              <Link className="mt-4 inline-block text-sm text-[var(--accent)]" to="/documents">前往文献库</Link>
            </div>
          ) : (
            <div className="space-y-3">
              {cards.map((card) => {
                const isEditing = editingCard?.id === card.id
                const isBusy = busyId === card.id

                return (
                  <article key={card.id} className="rounded-sm border border-[var(--line-muted)] bg-[rgba(248,251,247,0.74)] px-5 py-4">
                    <div className="grid gap-4 lg:grid-cols-[minmax(180px,0.8fr)_minmax(0,1.7fr)_180px_auto] lg:items-start">
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <h2 className="font-[var(--font-reading)] text-xl text-[var(--ink-strong)]">{card.term}</h2>
                          <span className={`rounded-full border px-2.5 py-0.5 text-xs ${statusClasses[card.mastery_status]}`}>{statusLabels[card.mastery_status]}</span>
                        </div>
                        {card.pronunciation ? <p className="mt-1 font-[var(--font-mono)] text-xs text-[var(--ink-soft)]">{card.pronunciation}</p> : null}
                      </div>
                      <div className="min-w-0">
                        <p className="text-sm leading-6 text-[var(--ink-main)]">{card.definition}</p>
                        {card.example_sentence ? <p className="mt-2 text-sm italic leading-6 text-[var(--ink-soft)]">{card.example_sentence}</p> : null}
                      </div>
                      <div className="text-xs leading-6 text-[var(--ink-soft)]">
                        <p>下次复习</p>
                        <p className="text-[var(--ink-main)]">{formatDate(card.next_review_at)}</p>
                        <Link className="mt-1 inline-block text-[var(--accent)]" to={`/documents/${card.document_id}`}>查看来源</Link>
                      </div>
                      <div className="flex gap-2 lg:justify-end">
                        <button disabled={isBusy} onClick={() => startEditing(card)} className="border border-[var(--line-strong)] px-3 py-1.5 text-xs transition hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:opacity-50">编辑</button>
                        <button disabled={isBusy} onClick={() => void handleDelete(card)} className="border border-[rgba(180,90,90,0.4)] px-3 py-1.5 text-xs text-[#8b3d3d] transition hover:bg-[#fff0f0] disabled:opacity-50">删除</button>
                      </div>
                    </div>

                    {isEditing ? (
                      <form onSubmit={handleUpdate} className="mt-4 grid gap-3 border-t border-[var(--line-muted)] pt-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_160px_auto]">
                        <textarea required value={definition} onChange={(event) => setDefinition(event.target.value)} rows={3} aria-label="释义" className="resize-y border border-[var(--line-muted)] bg-white/75 px-3 py-2 text-sm outline-none" />
                        <textarea value={exampleSentence} onChange={(event) => setExampleSentence(event.target.value)} rows={3} aria-label="例句" placeholder="例句（可选）" className="resize-y border border-[var(--line-muted)] bg-white/75 px-3 py-2 text-sm outline-none" />
                        <select aria-label="掌握状态" value={editStatus} onChange={(event) => setEditStatus(event.target.value as MasteryStatus)} className="h-10 border border-[var(--line-muted)] bg-white/75 px-3 text-sm outline-none">
                          {statusOptions.filter((option) => option.value !== 'all').map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                        </select>
                        <div className="flex gap-2 lg:justify-end">
                          <button type="submit" disabled={isBusy} className="h-10 bg-[#104f55] px-4 text-sm text-white disabled:opacity-50">保存</button>
                          <button type="button" onClick={() => setEditingCard(null)} className="h-10 border border-[var(--line-strong)] px-4 text-sm">取消</button>
                        </div>
                      </form>
                    ) : null}
                  </article>
                )
              })}
            </div>
          )}
        </section>

        <div className="mt-6 flex items-center justify-between border-t border-[var(--line-muted)] pt-4 text-sm">
          <button disabled={page <= 1 || isLoading} onClick={() => setPage((current) => current - 1)} className="border border-[var(--line-strong)] px-4 py-2 disabled:cursor-not-allowed disabled:opacity-40">上一页</button>
          <span className="text-[var(--ink-soft)]">第 {page} / {totalPages} 页</span>
          <button disabled={page >= totalPages || isLoading} onClick={() => setPage((current) => current + 1)} className="border border-[var(--line-strong)] px-4 py-2 disabled:cursor-not-allowed disabled:opacity-40">下一页</button>
        </div>
      </main>
    </div>
  )
}
