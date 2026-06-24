import { useEffect, useRef, useState, type ChangeEvent } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useGSAP } from '@gsap/react'
import gsap from 'gsap'
import { ScrollTrigger } from 'gsap/ScrollTrigger'
import { getDocument, GlobalWorkerOptions } from 'pdfjs-dist'
import { api, ApiError } from '../api/client'
import { useAuth } from '../auth/useAuth'
import type { DocumentDetail, DocumentListResponse, LearningOverview } from '../types'

GlobalWorkerOptions.workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString()

gsap.registerPlugin(ScrollTrigger, useGSAP)

const emptyStats: LearningOverview = {
  active_documents: 0,
  archived_documents: 0,
  paragraphs: 0,
  tags: 0,
  notes: 0,
  highlights: 0,
  tracked_documents: 0,
  average_progress_percent: 0,
}

function formatSource(sourceType: string) {
  if (sourceType === 'markdown') return 'Markdown'
  if (sourceType === 'txt') return 'TXT'
  return sourceType
}

function stripExtension(fileName: string) {
  return fileName.replace(/\.[^.]+$/, '')
}

async function extractPdfText(file: File) {
  const buffer = await file.arrayBuffer()
  const pdf = await getDocument({ data: buffer }).promise
  const pageTexts: string[] = []

  for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
    const page = await pdf.getPage(pageNumber)
    const textContent = await page.getTextContent()
    const items = textContent.items
      .map((item) => ('str' in item ? item.str : ''))
      .join(' ')
      .replace(/\s+/g, ' ')
      .trim()

    if (items) {
      pageTexts.push(items)
    }
  }

  return pageTexts.join('\n\n')
}

export function HomePage() {
  const navigate = useNavigate()
  const { user, logout } = useAuth()
  const fileInputRef = useRef<HTMLInputElement | null>(null)
  const scope = useRef<HTMLDivElement | null>(null)

  const [title, setTitle] = useState('')
  const [uploadHint, setUploadHint] = useState('可上传格式：.md / .markdown / .txt / .pdf')
  const [isUploading, setIsUploading] = useState(false)
  const [uploadError, setUploadError] = useState('')
  const [overview, setOverview] = useState<LearningOverview>(emptyStats)
  const [recentDocuments, setRecentDocuments] = useState<DocumentDetail[]>([])
  const [isLoadingData, setIsLoadingData] = useState(true)

  useGSAP(
    () => {
      const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
      if (reduceMotion) return

      const q = gsap.utils.selector(scope)
      const heroTimeline = gsap.timeline({
        defaults: { duration: 0.85, ease: 'power2.out' },
      })

      heroTimeline
        .from(q('[data-animate="nav"]'), { y: -18, opacity: 0, stagger: 0.08 })
        .from(q('[data-animate="sidebar"]'), { x: -20, opacity: 0 }, '-=0.45')
        .from(q('[data-animate="hero-shell"]'), { y: 24, opacity: 0, scale: 0.98 }, '-=0.42')
        .from(q('[data-animate="hero-title"]'), { y: 28, opacity: 0 }, '-=0.5')
        .from(q('[data-animate="hero-copy"]'), { y: 18, opacity: 0, stagger: 0.08 }, '-=0.42')
        .from(q('[data-layer]'), { y: 16, opacity: 0, scale: 0.96, stagger: 0.1 }, '-=0.58')

      q('[data-reveal]').forEach((element) => {
        gsap.fromTo(
          element,
          { y: 36, opacity: 0 },
          {
            y: 0,
            opacity: 1,
            duration: 0.9,
            ease: 'power2.out',
            scrollTrigger: {
              trigger: element,
              start: 'top 84%',
              once: true,
            },
          },
        )
      })
    },
    { scope },
  )

  useEffect(() => {
    let mounted = true

    const fetchData = async () => {
      setIsLoadingData(true)

      try {
        const [overviewData, recentList] = await Promise.all([
          api<LearningOverview>('/statistics/overview'),
          api<DocumentListResponse>('/documents?limit=3&offset=0'),
        ])

        if (!mounted) return
        setOverview(overviewData)

        const details = await Promise.all(
          recentList.items.map((item) => api<DocumentDetail>(`/documents/${item.id}`)),
        )

        if (mounted) {
          setRecentDocuments(details)
        }
      } catch (error) {
        if (mounted && error instanceof ApiError) {
          setUploadError(error.message)
        }
      } finally {
        if (mounted) {
          setIsLoadingData(false)
        }
      }
    }

    void fetchData()

    return () => {
      mounted = false
    }
  }, [])

  const handleFileChange = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    const lowerName = file.name.toLowerCase()
    const isPdf = lowerName.endsWith('.pdf')
    const validExtensions = ['.md', '.markdown', '.txt', '.pdf']
    const isSupported = validExtensions.some((extension) => lowerName.endsWith(extension))

    if (!isSupported) {
      setUploadError('当前支持 Markdown、TXT 和可提取文本的 PDF 文件。')
      setUploadHint('请上传 .md / .markdown / .txt / .pdf 文件')
      event.target.value = ''
      return
    }

    setUploadError('')
    setIsUploading(true)
    setUploadHint(`正在处理：${file.name}`)

    try {
      const uploadFile = isPdf
        ? await (async () => {
            setUploadHint(`正在解析 PDF：${file.name}`)
            const extractedText = await extractPdfText(file)

            if (!extractedText.trim()) {
              throw new Error('这个 PDF 没有可提取的文本，可能是扫描版图片 PDF。')
            }

            return new File([extractedText], `${stripExtension(file.name)}.txt`, {
              type: 'text/plain;charset=utf-8',
            })
          })()
        : file

      const formData = new FormData()
      formData.append('file', uploadFile)

      if (title.trim()) {
        formData.append('title', title.trim())
      } else if (isPdf) {
        formData.append('title', stripExtension(file.name))
      }

      const document = await api<DocumentDetail>('/documents', {
        method: 'POST',
        body: formData,
      })

      setUploadHint(`上传成功：${document.title}`)
      navigate(`/documents/${document.id}`)
    } catch (error) {
      if (error instanceof ApiError) {
        if (error.status === 413) {
          setUploadError('文件过大，请压缩后再试。')
        } else if (error.status === 415) {
          setUploadError('文件格式不支持，请上传 Markdown、TXT，或可提取文本的 PDF。')
        } else {
          setUploadError(error.message)
        }
      } else if (error instanceof Error) {
        setUploadError(error.message)
      } else {
        setUploadError('上传失败，请稍后再试。')
      }
    } finally {
      setIsUploading(false)
      event.target.value = ''
    }
  }

  const quickStats = [
    { label: '进行中文献', value: `${overview.active_documents} 篇` },
    { label: '已归档文献', value: `${overview.archived_documents} 篇` },
    { label: '平均阅读进度', value: `${overview.average_progress_percent.toFixed(1)}%` },
  ]

  const sidebarItems = [
    { label: '沉浸阅读', to: recentDocuments[0] ? `/documents/${recentDocuments[0].id}` : null },
    { label: '文献管理', to: '/documents' },
    { label: '标签与笔记', to: recentDocuments[0] ? `/documents/${recentDocuments[0].id}/notes-tags` : null },
  ]

  return (
    <div ref={scope} className="min-h-screen bg-[var(--bg-canvas)] text-[var(--ink-main)]">
      <div className="pointer-events-none fixed inset-0 bg-[radial-gradient(circle_at_top_left,_rgba(50,116,109,0.22),_transparent_22%),radial-gradient(circle_at_bottom_right,_rgba(158,197,171,0.18),_transparent_24%),linear-gradient(180deg,_rgba(223,234,226,0.95),_rgba(205,220,210,0.98))]" />

      <div className="relative mx-auto max-w-[1600px] px-6 py-6 lg:px-8">
        <header className="mb-6 flex flex-wrap items-center justify-between gap-6">
          <div className="flex flex-wrap items-baseline gap-6">
            <div data-animate="nav">
              <p className="font-[var(--font-ui)] text-xs tracking-[0.24em] text-[var(--ink-soft)]">项目</p>
              <p className="mt-1 font-[var(--font-display)] text-[1.9rem] leading-none text-[var(--ink-strong)]">IntelliRead</p>
            </div>
            <nav className="flex flex-wrap items-center gap-8 font-[var(--font-ui)] text-lg text-[var(--ink-strong)]">
              <Link data-animate="nav" className="transition hover:text-[var(--accent)]" to="/">首页</Link>
              <Link data-animate="nav" className="transition hover:text-[var(--accent)]" to="/documents">文献库</Link>
            </nav>
          </div>

          <div
            data-animate="nav"
            className="flex flex-wrap items-center gap-3 rounded-full border border-[var(--line-muted)] bg-white/70 px-4 py-2"
          >
            <div className="text-right">
              <p className="text-[11px] tracking-[0.16em] text-[var(--ink-soft)]">登录管理</p>
              <p className="text-sm text-[var(--ink-strong)]">{user?.username ?? '未登录'}</p>
            </div>
            <button
              onClick={logout}
              className="shrink-0 rounded-full border border-[#104f55] bg-[#104f55] px-4 py-2 text-sm text-white transition hover:bg-[#0d4348]"
            >
              退出登录
            </button>
          </div>
        </header>

        <main className="border-frame rounded-[4px] bg-[rgba(244,248,244,0.56)] p-2 shadow-[var(--shadow-soft)]">
          <div className="grid min-h-[640px] grid-cols-[130px_minmax(0,1fr)] gap-8 border border-[rgba(50,116,109,0.9)] px-2 py-3 lg:px-3">
            <aside data-animate="sidebar" className="border border-[rgba(50,116,109,0.9)] bg-[rgba(229,239,232,0.72)] px-4 py-6">
              <div className="flex h-full flex-col justify-between">
                <div className="space-y-8">
                  {sidebarItems.map((item) =>
                    item.to ? (
                      <Link
                        key={item.label}
                        to={item.to}
                        className="block text-left font-[var(--font-reading)] text-[1.05rem] leading-10 text-[var(--ink-strong)] transition hover:text-[var(--accent)]"
                      >
                        {item.label}
                      </Link>
                    ) : (
                      <div
                        key={item.label}
                        className="block text-left font-[var(--font-reading)] text-[1.05rem] leading-10 text-[var(--ink-soft)]"
                      >
                        {item.label}
                      </div>
                    ),
                  )}
                </div>

                <div className="rounded-sm border border-[rgba(50,116,109,0.68)] bg-[rgba(255,255,255,0.42)] px-3 py-3 font-[var(--font-ui)] text-sm text-[var(--ink-soft)]">
                  当前目标
                  <div className="mt-2 font-[var(--font-reading)] text-base text-[var(--ink-strong)]">减少查词打断，沉浸完成阅读</div>
                </div>
              </div>
            </aside>

            <section className="grid grid-rows-[1fr_auto] gap-14 px-4 py-8 lg:px-10">
              <div className="grid items-start gap-8 xl:grid-cols-[minmax(260px,300px)_minmax(0,1fr)]">
                <div data-reveal className="border border-[rgba(50,116,109,0.9)] bg-[rgba(236,243,237,0.8)] p-5 shadow-[var(--shadow-soft)]">
                  <p className="font-[var(--font-ui)] text-sm tracking-[0.18em] text-[var(--ink-soft)]">学习概览</p>
                  <div className="mt-5 space-y-4">
                    {quickStats.map((item) => (
                      <div key={item.label} className="border-b border-[var(--line-muted)] pb-4 last:border-b-0 last:pb-0">
                        <p className="font-[var(--font-ui)] text-sm text-[var(--ink-soft)]">{item.label}</p>
                        <p className="mt-1 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">{item.value}</p>
                      </div>
                    ))}
                  </div>
                  <div className="mt-6 rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.36)] p-4">
                    <p className="font-[var(--font-reading)] text-lg text-[var(--ink-strong)]">当前数据</p>
                    <p className="mt-2 font-[var(--font-ui)] text-sm leading-7 text-[var(--ink-soft)]">
                      已累计 {overview.paragraphs} 个段落，{overview.tags} 个标签，{overview.notes} 条笔记，{overview.highlights} 条高亮。
                    </p>
                  </div>
                </div>

                <div className="flex items-start justify-center pt-10 xl:pt-14">
                  <div data-animate="hero-shell" className="homepage-hero-shell relative w-full max-w-[760px] border border-[rgba(50,116,109,0.9)] bg-[rgba(242,245,238,0.78)] px-10 py-20 text-center shadow-[var(--shadow-paper)]">
                    <div data-layer="back" className="hero-layer absolute right-4 top-8 hidden h-28 w-40 rounded-[22px] border border-[rgba(16,79,85,0.16)] bg-[linear-gradient(180deg,rgba(158,197,171,0.55),rgba(255,255,255,0.5))] lg:block">
                      <div className="p-4 text-left">
                        <div className="h-2 w-14 rounded-full bg-[rgba(16,79,85,0.14)]" />
                        <div className="mt-4 h-2 w-24 rounded-full bg-[rgba(16,79,85,0.1)]" />
                        <div className="mt-2 h-2 w-18 rounded-full bg-[rgba(16,79,85,0.08)]" />
                      </div>
                    </div>
                    <div data-layer="mid" className="hero-layer absolute left-4 bottom-8 hidden h-24 w-32 rounded-[20px] border border-[rgba(16,79,85,0.18)] bg-[linear-gradient(180deg,rgba(255,255,255,0.86),rgba(228,237,231,0.72))] lg:block">
                      <div className="p-4 text-left">
                        <div className="font-[var(--font-mono)] text-[0.62rem] uppercase tracking-[0.16em] text-[var(--ink-soft)]">READ</div>
                        <div className="mt-3 h-2 w-16 rounded-full bg-[rgba(16,79,85,0.12)]" />
                        <div className="mt-2 h-2 w-11 rounded-full bg-[rgba(16,79,85,0.08)]" />
                      </div>
                    </div>
                    <div data-layer="front" className="hero-layer absolute right-4 top-1/2 hidden h-36 w-44 -translate-y-1/2 rounded-[24px] border border-[rgba(16,79,85,0.2)] bg-[rgba(248,251,247,0.92)] p-4 shadow-[0_14px_32px_rgba(16,79,85,0.12)] lg:block">
                      <div className="text-left">
                        <p className="font-[var(--font-ui)] text-xs tracking-[0.16em] text-[var(--ink-soft)]">阅读流程</p>
                        <div className="mt-4 space-y-2">
                          <div className="rounded-full bg-[rgba(50,116,109,0.14)] px-3 py-1 text-sm text-[var(--ink-strong)]">导入文献</div>
                          <div className="rounded-full bg-[rgba(158,197,171,0.22)] px-3 py-1 text-sm text-[var(--ink-strong)]">标注段落</div>
                          <div className="rounded-full bg-[rgba(16,79,85,0.08)] px-3 py-1 text-sm text-[var(--ink-strong)]">进入复习</div>
                        </div>
                      </div>
                    </div>

                    <div className="mx-auto max-w-[36rem] lg:pr-32">
                      <p data-animate="hero-copy" className="mb-4 font-[var(--font-ui)] text-sm tracking-[0.18em] text-[var(--ink-soft)]">首页概览</p>
                      <h1 data-animate="hero-title" className="font-[var(--font-display)] text-6xl leading-[1.06] text-[var(--ink-strong)]">让阅读停留在理解本身</h1>
                      <p data-animate="hero-copy" className="mx-auto mt-6 font-[var(--font-ui)] text-base leading-8 text-[var(--ink-soft)]">
                        IntelliRead 帮助用户在阅读外语长文时减少查词打断，并把生词自动纳入后续复习流程。
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="grid gap-8 xl:grid-cols-[minmax(0,1.05fr)_minmax(0,1.2fr)]">
                <div data-reveal className="border border-[rgba(50,116,109,0.9)] bg-[rgba(242,245,238,0.76)] px-8 py-10 shadow-[var(--shadow-soft)]">
                  <input
                    ref={fileInputRef}
                    type="file"
                    accept=".md,.markdown,.txt,.pdf,text/plain,text/markdown,application/pdf"
                    onChange={handleFileChange}
                    className="hidden"
                  />
                  <p className="font-[var(--font-reading)] text-2xl leading-10 text-[var(--ink-strong)]">从这里开始导入文献</p>
                  <p className="mt-4 font-[var(--font-ui)] text-sm leading-7 text-[var(--ink-soft)]">
                    当前支持 Markdown、TXT，以及可提取文本的 PDF。PDF 会先在浏览器中解析文字，再自动进入沉浸式阅读页。
                  </p>
                  <label className="mt-5 block">
                    <span className="mb-2 block text-sm text-[var(--ink-soft)]">文献标题（可选）</span>
                    <input
                      type="text"
                      value={title}
                      onChange={(event) => setTitle(event.target.value)}
                      className="w-full rounded-[18px] border border-[var(--line-muted)] bg-white/60 px-4 py-3 outline-none"
                      placeholder="例如：AQA 论文导读"
                    />
                  </label>
                  <div className="mt-5 rounded-sm border border-dashed border-[var(--line-strong)] bg-[rgba(255,255,255,0.34)] px-4 py-4 font-[var(--font-ui)] text-sm leading-7 text-[var(--ink-soft)]">
                    {uploadHint}
                  </div>
                  {uploadError ? (
                    <div className="mt-4 rounded-[18px] border border-[rgba(180,90,90,0.22)] bg-[rgba(255,240,240,0.7)] px-4 py-3 text-sm text-[#8b3d3d]">{uploadError}</div>
                  ) : null}
                  <button
                    onClick={() => fileInputRef.current?.click()}
                    disabled={isUploading}
                    className="mt-6 rounded-full border border-[var(--line-strong)] px-5 py-2.5 font-[var(--font-ui)] text-sm text-[var(--ink-main)] transition hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60"
                  >
                    {isUploading ? '处理中...' : '选择文件'}
                  </button>
                </div>

                <div data-reveal className="grid gap-6 border border-[rgba(50,116,109,0.9)] bg-[rgba(229,239,232,0.76)] px-6 py-6 shadow-[var(--shadow-soft)] lg:grid-cols-[minmax(0,1fr)_220px]">
                  <div>
                    <p className="font-[var(--font-ui)] text-sm tracking-[0.16em] text-[var(--ink-soft)]">最近文献</p>
                    <div className="mt-5 space-y-4">
                      {isLoadingData ? (
                        <div className="text-sm text-[var(--ink-soft)]">正在加载文献数据...</div>
                      ) : recentDocuments.length === 0 ? (
                        <div className="text-sm leading-7 text-[var(--ink-soft)]">还没有文献，导入第一篇之后这里会显示最近阅读内容。</div>
                      ) : (
                        recentDocuments.map((doc) => (
                          <Link key={doc.id} to={`/documents/${doc.id}`} className="block border-b border-[var(--line-muted)] pb-3 last:border-b-0 last:pb-0">
                            <p className="font-[var(--font-reading)] text-lg text-[var(--ink-strong)]">{doc.title}</p>
                            <p className="mt-1 font-[var(--font-ui)] text-sm text-[var(--ink-soft)]">{formatSource(doc.source_type)} · {doc.paragraphs.length} 段</p>
                          </Link>
                        ))
                      )}
                    </div>
                  </div>
                  <div className="rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.36)] p-5">
                    <p className="font-[var(--font-ui)] text-sm tracking-[0.16em] text-[var(--ink-soft)]">当前节奏</p>
                    <div className="mt-4 space-y-3">
                      <div className="rounded-full bg-[rgba(50,116,109,0.12)] px-3 py-2 font-[var(--font-ui)] text-sm text-[var(--ink-strong)]">已跟踪 {overview.tracked_documents} 篇阅读进度</div>
                      <div className="rounded-full bg-[rgba(158,197,171,0.18)] px-3 py-2 font-[var(--font-ui)] text-sm text-[var(--ink-strong)]">已积累 {overview.notes} 条阅读笔记</div>
                      <div className="rounded-full bg-[rgba(16,79,85,0.08)] px-3 py-2 font-[var(--font-ui)] text-sm text-[var(--ink-strong)]">已建立 {overview.highlights} 条重点高亮</div>
                    </div>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </main>
      </div>
    </div>
  )
}
