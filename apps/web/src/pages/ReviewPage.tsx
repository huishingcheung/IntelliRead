import { useCallback, useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { ApiError } from '../api/client'
import { getReviewQueue, submitReviewAnswer } from '../api/vocabulary'
import { useAuth } from '../auth/useAuth'
import type { ReviewAnswer, ReviewResult, VocabularyCard } from '../types'

const answerOptions: Array<{
  value: ReviewResult
  label: string
  interval: string
  className: string
}> = [
  { value: 'wrong', label: '没记住', interval: '10 分钟', className: 'border-[#9b4b4b] text-[#873f3f] hover:bg-[#fff0f0]' },
  { value: 'hard', label: '有点难', interval: '1 天', className: 'border-[#a16d2d] text-[#80551f] hover:bg-[#f8efe2]' },
  { value: 'good', label: '记住了', interval: '3 天', className: 'border-[#32746d] text-[#245c57] hover:bg-[#e3efeb]' },
  { value: 'easy', label: '很熟悉', interval: '7 天', className: 'border-[#557b48] text-[#426438] hover:bg-[#e7f0e3]' },
]

function formatDate(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

export function ReviewPage() {
  const { user, logout } = useAuth()
  const [queue, setQueue] = useState<VocabularyCard[]>([])
  const [initialTotal, setInitialTotal] = useState(0)
  const [reviewedCount, setReviewedCount] = useState(0)
  const [isLoading, setIsLoading] = useState(true)
  const [isRevealed, setIsRevealed] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState('')
  const [latestAnswer, setLatestAnswer] = useState<ReviewAnswer | null>(null)

  const loadQueue = useCallback(async () => {
    setIsLoading(true)
    setError('')

    try {
      const cards = await getReviewQueue(100)
      setQueue(cards)
      setInitialTotal(cards.length)
      setReviewedCount(0)
      setIsRevealed(false)
      setLatestAnswer(null)
    } catch (loadError) {
      setError(loadError instanceof ApiError ? loadError.message : '加载复习队列失败。')
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void loadQueue()
    }, 0)

    return () => window.clearTimeout(timer)
  }, [loadQueue])

  const currentCard = queue[0] ?? null
  const completedPercent = useMemo(
    () => (initialTotal === 0 ? 0 : (reviewedCount / initialTotal) * 100),
    [initialTotal, reviewedCount],
  )

  const handleAnswer = async (result: ReviewResult) => {
    if (!currentCard || isSubmitting) return

    setIsSubmitting(true)
    setError('')

    try {
      const answer = await submitReviewAnswer(currentCard.id, result)
      setLatestAnswer(answer)
      setQueue((current) => current.filter((card) => card.id !== currentCard.id))
      setReviewedCount((current) => current + 1)
      setIsRevealed(false)
    } catch (answerError) {
      setError(answerError instanceof ApiError ? answerError.message : '提交复习结果失败。')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] text-[var(--ink-main)]">
      <header className="border-b border-[var(--line-muted)] bg-[rgba(244,248,244,0.82)] px-6 py-4">
        <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4">
          <nav className="flex flex-wrap items-center gap-6 text-sm text-[var(--ink-strong)]">
            <Link className="font-[var(--font-display)] text-xl" to="/">IntelliRead</Link>
            <Link className="transition hover:text-[var(--accent)]" to="/documents">文献库</Link>
            <Link className="transition hover:text-[var(--accent)]" to="/vocabulary">生词本</Link>
            <Link className="font-medium text-[var(--accent)]" to="/review">复习</Link>
          </nav>
          <div className="flex items-center gap-3 text-sm">
            <span className="text-[var(--ink-soft)]">{user?.username}</span>
            <button onClick={logout} className="border border-[var(--line-strong)] px-3 py-1.5 transition hover:border-[var(--accent)] hover:text-[var(--accent)]">退出</button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-6xl px-6 py-8">
        <div className="grid gap-8 lg:grid-cols-[220px_minmax(0,1fr)]">
          <aside className="border-r border-[var(--line-strong)] pr-6">
            <p className="text-sm tracking-[0.18em] text-[var(--ink-soft)]">REVIEW</p>
            <h1 className="mt-2 font-[var(--font-reading)] text-3xl text-[var(--ink-strong)]">复习队列</h1>
            <dl className="mt-8 space-y-5 text-sm">
              <div>
                <dt className="text-[var(--ink-soft)]">本轮总数</dt>
                <dd className="mt-1 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">{initialTotal}</dd>
              </div>
              <div>
                <dt className="text-[var(--ink-soft)]">已经完成</dt>
                <dd className="mt-1 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">{reviewedCount}</dd>
              </div>
              <div>
                <dt className="text-[var(--ink-soft)]">剩余</dt>
                <dd className="mt-1 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">{queue.length}</dd>
              </div>
            </dl>
            <div className="mt-8 h-2 overflow-hidden bg-[rgba(16,79,85,0.12)]">
              <div className="h-full bg-[#32746d] transition-[width] duration-300" style={{ width: `${completedPercent}%` }} />
            </div>
            <button disabled={isLoading} onClick={() => void loadQueue()} className="mt-8 w-full border border-[var(--line-strong)] px-4 py-2 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:opacity-50">刷新队列</button>
          </aside>

          <section className="min-h-[620px]">
            {error ? <div className="mb-5 border border-[rgba(180,90,90,0.3)] bg-[#fff0f0] px-4 py-3 text-sm text-[#8b3d3d]">{error}</div> : null}
            {latestAnswer ? (
              <div className="mb-5 flex flex-wrap items-center justify-between gap-3 border-b border-[var(--line-muted)] pb-4 text-sm">
                <span className="text-[var(--ink-main)]">已记录：{answerOptions.find((option) => option.value === latestAnswer.answer_result)?.label}</span>
                <span className="text-[var(--ink-soft)]">下次复习 {formatDate(latestAnswer.next_review_at)}</span>
              </div>
            ) : null}

            {isLoading ? (
              <div className="flex min-h-[520px] items-center justify-center text-[var(--ink-soft)]">正在准备复习内容...</div>
            ) : !currentCard ? (
              <div className="flex min-h-[520px] flex-col items-center justify-center border border-dashed border-[var(--line-strong)] px-6 text-center">
                <p className="font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">
                  {initialTotal > 0 && reviewedCount >= initialTotal ? '本轮复习已完成' : '当前没有待复习词汇'}
                </p>
                <div className="mt-6 flex flex-wrap justify-center gap-3">
                  <Link to="/vocabulary" className="border border-[var(--line-strong)] px-5 py-2.5 text-sm">查看生词本</Link>
                  <Link to="/documents" className="bg-[#104f55] px-5 py-2.5 text-sm text-white">继续阅读</Link>
                </div>
              </div>
            ) : (
              <article className="rounded-sm border border-[var(--line-strong)] bg-[rgba(248,251,247,0.78)] px-8 py-8">
                <div className="flex flex-wrap items-start justify-between gap-4 border-b border-[var(--line-muted)] pb-5">
                  <div>
                    <p className="text-xs tracking-[0.16em] text-[var(--ink-soft)]">{reviewedCount + 1} / {initialTotal}</p>
                    <h2 className="mt-3 font-[var(--font-reading)] text-4xl text-[var(--ink-strong)]">{currentCard.term}</h2>
                    {currentCard.pronunciation ? <p className="mt-2 font-[var(--font-mono)] text-sm text-[var(--ink-soft)]">{currentCard.pronunciation}</p> : null}
                  </div>
                  <Link to={`/documents/${currentCard.document_id}`} className="text-sm text-[var(--accent)]">查看来源文献</Link>
                </div>

                <div className="min-h-[250px] py-8">
                  {currentCard.source_text ? (
                    <blockquote className="border-l-2 border-[var(--accent)] pl-4 text-base leading-8 text-[var(--ink-soft)]">{currentCard.source_text}</blockquote>
                  ) : null}

                  {isRevealed ? (
                    <div className="mt-8 border-t border-[var(--line-muted)] pt-6">
                      <p className="text-xs tracking-[0.16em] text-[var(--ink-soft)]">释义</p>
                      <p className="mt-3 text-lg leading-8 text-[var(--ink-strong)]">{currentCard.definition}</p>
                      {currentCard.example_sentence ? (
                        <p className="mt-4 text-sm italic leading-7 text-[var(--ink-soft)]">{currentCard.example_sentence}</p>
                      ) : null}
                    </div>
                  ) : (
                    <div className="mt-10 flex justify-center">
                      <button onClick={() => setIsRevealed(true)} className="bg-[#104f55] px-7 py-3 text-sm font-medium text-white transition hover:bg-[#0d4348]">显示释义</button>
                    </div>
                  )}
                </div>

                <div className="grid min-h-[76px] grid-cols-2 gap-3 border-t border-[var(--line-muted)] pt-5 lg:grid-cols-4">
                  {answerOptions.map((option) => (
                    <button key={option.value} disabled={!isRevealed || isSubmitting} onClick={() => void handleAnswer(option.value)} className={`border px-4 py-3 text-left transition disabled:cursor-not-allowed disabled:opacity-35 ${option.className}`}>
                      <span className="block text-sm font-medium">{option.label}</span>
                      <span className="mt-1 block text-xs opacity-75">{option.interval}后</span>
                    </button>
                  ))}
                </div>
              </article>
            )}
          </section>
        </div>
      </main>
    </div>
  )
}
