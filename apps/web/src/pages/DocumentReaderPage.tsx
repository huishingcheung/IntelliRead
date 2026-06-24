import { useCallback, useEffect, useMemo, useRef, useState, type MouseEvent } from 'react'
import { Link, useParams } from 'react-router-dom'
import { api, ApiError } from '../api/client'
import { useAuth } from '../auth/useAuth'
import type {
  DocumentDetail,
  DocumentProgress,
  Highlight,
  HighlightColor,
  Note,
  Paragraph,
  Tag,
} from '../types'

const highlightColors: HighlightColor[] = ['yellow', 'green', 'blue', 'pink', 'purple']

const colorClassMap: Record<HighlightColor, string> = {
  yellow: 'bg-[#f4e7a1]',
  green: 'bg-[#cfe5c8]',
  blue: 'bg-[#cfe0ec]',
  pink: 'bg-[#edd3de]',
  purple: 'bg-[#ddd4ec]',
}

const colorLabelMap: Record<HighlightColor, string> = {
  yellow: '黄色',
  green: '绿色',
  blue: '蓝色',
  pink: '粉色',
  purple: '紫色',
}

const colorDotClassMap: Record<HighlightColor, string> = {
  yellow: 'bg-[#d7b640]',
  green: 'bg-[#5d8f61]',
  blue: 'bg-[#6487a8]',
  pink: 'bg-[#bb7891]',
  purple: 'bg-[#8c79b5]',
}

type PendingSelection = {
  text: string
  paragraph: Paragraph
  startOffset: number
  endOffset: number
}

type SelectionPopover = {
  x: number
  y: number
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

function buildDemoTranslation(text: string) {
  if (!text) return ''
  const preview = text.length > 36 ? `${text.slice(0, 36)}...` : text
  return `演示翻译：${preview}。这里先保留前端交互，后续可以直接接入真实翻译接口。`
}

function buildDemoAnalysis(text: string) {
  if (!text) return []
  return [
    '这段文本可能包含专业术语，建议结合上下文理解。',
    '长句可以先找主语和谓语，再处理定语和从句层级。',
    '当前为前端演示分析卡，后续可接入真实 AI 解析接口。',
  ]
}

function getCharLength(text: string) {
  return Array.from(text).length
}

function sliceChars(text: string, start: number, end?: number) {
  return Array.from(text).slice(start, end).join('')
}

function applyHighlights(content: string, paragraphId: string, highlights: Highlight[]) {
  const related = highlights
    .filter((item) => item.paragraph_id === paragraphId)
    .sort((a, b) => {
      if (a.start_offset !== b.start_offset) return a.start_offset - b.start_offset
      if (a.end_offset !== b.end_offset) return a.end_offset - b.end_offset
      return new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime()
    })

  if (related.length === 0) {
    return [{ text: content, color: null as HighlightColor | null }]
  }

  const chars = Array.from(content)
  const colorByChar = new Array<HighlightColor | null>(chars.length).fill(null)

  for (const highlight of related) {
    const start = Math.max(0, highlight.start_offset)
    const end = Math.min(chars.length, highlight.end_offset)

    for (let index = start; index < end; index += 1) {
      colorByChar[index] = highlight.color
    }
  }

  const segments: Array<{ text: string; color: HighlightColor | null }> = []
  let currentText = ''
  let currentColor = colorByChar[0] ?? null

  for (let index = 0; index < chars.length; index += 1) {
    const nextColor = colorByChar[index] ?? null
    if (index === 0 || nextColor === currentColor) {
      currentText += chars[index]
      continue
    }

    segments.push({ text: currentText, color: currentColor })
    currentText = chars[index]
    currentColor = nextColor
  }

  if (currentText) {
    segments.push({ text: currentText, color: currentColor })
  }

  return segments
}

function getSelectionOffsets(article: HTMLElement, range: Range) {
  const preRange = range.cloneRange()
  preRange.selectNodeContents(article)
  preRange.setEnd(range.startContainer, range.startOffset)

  const selectedText = range.toString()
  const startOffset = getCharLength(preRange.toString())
  const selectedLength = getCharLength(selectedText)

  return {
    text: selectedText.trim(),
    startOffset,
    endOffset: startOffset + selectedLength,
  }
}

export function DocumentReaderPage() {
  const { id = '' } = useParams()
  const { user, logout } = useAuth()
  const contentRef = useRef<HTMLDivElement | null>(null)
  const selectionToolsRef = useRef<HTMLDivElement | null>(null)
  const paragraphRefs = useRef<Array<HTMLElement | null>>([])
  const hasInitializedProgress = useRef(false)

  const [document, setDocument] = useState<DocumentDetail | null>(null)
  const [progress, setProgress] = useState<DocumentProgress | null>(null)
  const [allTags, setAllTags] = useState<Tag[]>([])
  const [documentTags, setDocumentTags] = useState<Tag[]>([])
  const [notes, setNotes] = useState<Note[]>([])
  const [highlights, setHighlights] = useState<Highlight[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [activeParagraphIndex, setActiveParagraphIndex] = useState(0)
  const [newTagName, setNewTagName] = useState('')
  const [selectedTagIds, setSelectedTagIds] = useState<string[]>([])
  const [noteContent, setNoteContent] = useState('')
  const [noteParagraphId, setNoteParagraphId] = useState('document')
  const [selection, setSelection] = useState<PendingSelection | null>(null)
  const [selectionPopover, setSelectionPopover] = useState<SelectionPopover | null>(null)
  const [highlightColor, setHighlightColor] = useState<HighlightColor>('yellow')
  const [isSavingProgress, setIsSavingProgress] = useState(false)
  const [panelMessage, setPanelMessage] = useState('')

  const activeHighlight = useMemo(() => {
    if (!selection) return null
    return (
      highlights.find(
        (item) =>
          item.paragraph_id === selection.paragraph.id &&
          item.start_offset === selection.startOffset &&
          item.end_offset === selection.endOffset,
      ) ?? null
    )
  }, [highlights, selection])

  const fetchReaderData = useCallback(async () => {
    setIsLoading(true)
    setError('')

    try {
      const [documentData, progressData, tagData, attachedTags, notesData, highlightsData] =
        await Promise.all([
          api<DocumentDetail>(`/documents/${id}`),
          api<DocumentProgress | null>(`/documents/${id}/progress`),
          api<Tag[]>('/tags'),
          api<Tag[]>(`/documents/${id}/tags`),
          api<Note[]>(`/documents/${id}/notes`),
          api<Highlight[]>(`/documents/${id}/highlights`),
        ])

      setDocument(documentData)
      setProgress(progressData)
      setAllTags(tagData)
      setDocumentTags(attachedTags)
      setSelectedTagIds(attachedTags.map((item) => item.id))
      setNotes(notesData)
      setHighlights(highlightsData)

      const nextIndex = progressData?.paragraph_position ?? 0
      setActiveParagraphIndex(Math.max(0, Math.min(nextIndex, documentData.paragraphs.length - 1)))
      hasInitializedProgress.current = true
    } catch (fetchError) {
      setError(fetchError instanceof ApiError ? fetchError.message : '加载文献失败。')
    } finally {
      setIsLoading(false)
    }
  }, [id])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void fetchReaderData()
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [fetchReaderData])

  const readingPercent = useMemo(() => {
    if (!document) return 0
    return Number((((activeParagraphIndex + 1) / document.paragraphs.length) * 100).toFixed(1))
  }, [activeParagraphIndex, document])

  const effectiveParagraphIndex = useMemo(() => {
    return Math.max(activeParagraphIndex, progress?.paragraph_position ?? 0)
  }, [activeParagraphIndex, progress?.paragraph_position])

  const effectiveProgressPercent = useMemo(() => {
    return Math.max(readingPercent, progress?.progress_percent ?? 0)
  }, [progress?.progress_percent, readingPercent])

  useEffect(() => {
    if (!document || !hasInitializedProgress.current) return

    if (
      progress &&
      progress.paragraph_position === effectiveParagraphIndex &&
      Math.abs(progress.progress_percent - effectiveProgressPercent) < 0.1
    ) {
      return
    }

    const timer = window.setTimeout(async () => {
      try {
        setIsSavingProgress(true)
        const nextProgress = await api<DocumentProgress>(`/documents/${id}/progress`, {
          method: 'PUT',
          body: JSON.stringify({
            paragraph_position: effectiveParagraphIndex,
            progress_percent: effectiveProgressPercent,
          }),
        })
        setProgress(nextProgress)
      } catch (saveError) {
        if (saveError instanceof ApiError) {
          setPanelMessage(saveError.message)
        }
      } finally {
        setIsSavingProgress(false)
      }
    }, 350)

    return () => {
      window.clearTimeout(timer)
    }
  }, [document, effectiveParagraphIndex, effectiveProgressPercent, id, progress])

  useEffect(() => {
    if (!document || document.paragraphs.length === 0) return

    const observer = new IntersectionObserver(
      (entries) => {
        const visibleEntries = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio)

        if (visibleEntries.length === 0) return

        const nextParagraphId = visibleEntries[0].target.getAttribute('data-paragraph-id')
        if (!nextParagraphId) return

        const nextIndex = document.paragraphs.findIndex((item) => item.id === nextParagraphId)
        if (nextIndex >= 0) {
          setActiveParagraphIndex((current) => (current === nextIndex ? current : nextIndex))
        }
      },
      {
        root: null,
        rootMargin: '-18% 0px -45% 0px',
        threshold: [0.2, 0.35, 0.5, 0.7],
      },
    )

    const nodes = paragraphRefs.current.filter((node): node is HTMLElement => Boolean(node))
    nodes.forEach((node) => observer.observe(node))

    return () => {
      observer.disconnect()
    }
  }, [document])

  useEffect(() => {
    const handleWindowSelectionClear = () => {
      const activeElement = window.document.activeElement
      if (
        activeElement instanceof HTMLElement &&
        selectionToolsRef.current?.contains(activeElement)
      ) {
        return
      }

      const nativeSelection = window.getSelection()
      if (!nativeSelection || !nativeSelection.toString().trim()) {
        setSelectionPopover(null)
      }
    }

    window.document.addEventListener('selectionchange', handleWindowSelectionClear)
    return () => {
      window.document.removeEventListener('selectionchange', handleWindowSelectionClear)
    }
  }, [])

  const handleSelectParagraph = (index: number) => {
    setSelection(null)
    setSelectionPopover(null)
    setActiveParagraphIndex(index)
    paragraphRefs.current[index]?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }

  const clearSelection = () => {
    setSelection(null)
    setSelectionPopover(null)
    window.getSelection()?.removeAllRanges()
  }

  const handleTextSelection = () => {
    if (!document || !contentRef.current) return

    const nativeSelection = window.getSelection()
    if (!nativeSelection || nativeSelection.rangeCount === 0) {
      setSelection(null)
      setSelectionPopover(null)
      return
    }

    const rawText = nativeSelection.toString()
    if (!rawText.trim()) {
      setSelection(null)
      setSelectionPopover(null)
      return
    }

    const range = nativeSelection.getRangeAt(0)
    const startElement = range.startContainer instanceof Element ? range.startContainer : range.startContainer.parentElement
    const endElement = range.endContainer instanceof Element ? range.endContainer : range.endContainer.parentElement

    const startArticle = startElement?.closest('article[data-paragraph-id]') as HTMLElement | null
    const endArticle = endElement?.closest('article[data-paragraph-id]') as HTMLElement | null

    if (!startArticle || !endArticle || startArticle !== endArticle) {
      setSelection(null)
      setSelectionPopover(null)
      setPanelMessage('暂不支持跨段落高亮，请在单个段落内选择文本。')
      return
    }

    const paragraphId = startArticle.getAttribute('data-paragraph-id')
    if (!paragraphId) {
      setSelection(null)
      setSelectionPopover(null)
      return
    }

    const paragraph = document.paragraphs.find((item) => item.id === paragraphId)
    if (!paragraph) {
      setSelection(null)
      setSelectionPopover(null)
      return
    }

    const offsets = getSelectionOffsets(startArticle, range)
    if (!offsets.text || offsets.endOffset <= offsets.startOffset) {
      setSelection(null)
      setSelectionPopover(null)
      return
    }

    const rect = range.getBoundingClientRect()
    const containerRect = contentRef.current.getBoundingClientRect()

    setSelection({
      text: offsets.text,
      paragraph,
      startOffset: offsets.startOffset,
      endOffset: offsets.endOffset,
    })
    setSelectionPopover({
      x: rect.left - containerRect.left,
      y: rect.top - containerRect.top - 78,
    })
    setActiveParagraphIndex(paragraph.position)
    setPanelMessage('已选中文本。你可以直接用浮层按钮加入高亮，也可以在右侧面板继续操作。')
  }

  const handleCreateTag = async () => {
    if (!newTagName.trim()) return

    try {
      const tag = await api<Tag>('/tags', {
        method: 'POST',
        body: JSON.stringify({ name: newTagName.trim() }),
      })
      setAllTags((current) => [...current, tag].sort((a, b) => a.name.localeCompare(b.name, 'zh-CN')))
      setSelectedTagIds((current) => [...current, tag.id])
      setNewTagName('')
      setPanelMessage(`已创建标签：${tag.name}`)
    } catch (tagError) {
      setPanelMessage(tagError instanceof ApiError ? tagError.message : '创建标签失败。')
    }
  }

  const handleSaveTags = async () => {
    try {
      const nextTags = await api<Tag[]>(`/documents/${id}/tags`, {
        method: 'PUT',
        body: JSON.stringify({ tag_ids: selectedTagIds }),
      })
      setDocumentTags(nextTags)
      setPanelMessage('文献标签已更新。')
    } catch (tagError) {
      setPanelMessage(tagError instanceof ApiError ? tagError.message : '保存标签失败。')
    }
  }

  const handleCreateNote = async () => {
    if (!noteContent.trim()) return

    try {
      const note = await api<Note>(`/documents/${id}/notes`, {
        method: 'POST',
        body: JSON.stringify({
          content: noteContent.trim(),
          paragraph_id: noteParagraphId === 'document' ? null : noteParagraphId,
        }),
      })
      setNotes((current) => [note, ...current])
      setNoteContent('')
      setNoteParagraphId('document')
      setPanelMessage('笔记已添加。')
    } catch (noteError) {
      setPanelMessage(noteError instanceof ApiError ? noteError.message : '添加笔记失败。')
    }
  }

  const handleDeleteNote = async (noteId: string) => {
    try {
      await api(`/notes/${noteId}`, { method: 'DELETE' })
      setNotes((current) => current.filter((note) => note.id !== noteId))
    } catch (noteError) {
      setPanelMessage(noteError instanceof ApiError ? noteError.message : '删除笔记失败。')
    }
  }

  const handleCreateHighlight = async () => {
    if (!selection) {
      setPanelMessage('请先在左侧正文里用鼠标左键拖选一段文字。')
      return
    }

    try {
      if (activeHighlight) {
        const updatedHighlight = await api<Highlight>(`/highlights/${activeHighlight.id}`, {
          method: 'PUT',
          body: JSON.stringify({
            paragraph_id: selection.paragraph.id,
            start_offset: selection.startOffset,
            end_offset: selection.endOffset,
            color: highlightColor,
          }),
        })
        setHighlights((current) =>
          current.map((item) => (item.id === updatedHighlight.id ? updatedHighlight : item)),
        )
        setPanelMessage(`高亮颜色已更新为${colorLabelMap[highlightColor]}。`)
      } else {
        const highlight = await api<Highlight>(`/documents/${id}/highlights`, {
          method: 'POST',
          body: JSON.stringify({
            paragraph_id: selection.paragraph.id,
            start_offset: selection.startOffset,
            end_offset: selection.endOffset,
            color: highlightColor,
          }),
        })
        setHighlights((current) => [highlight, ...current])
        setPanelMessage('高亮已保存。')
      }
      clearSelection()
    } catch (highlightError) {
      setPanelMessage(highlightError instanceof ApiError ? highlightError.message : '创建高亮失败。')
    }
  }

  const handleDeleteHighlight = async (highlightId: string) => {
    try {
      await api(`/highlights/${highlightId}`, { method: 'DELETE' })
      setHighlights((current) => current.filter((item) => item.id !== highlightId))
      if (activeHighlight?.id === highlightId) {
        clearSelection()
      }
      setPanelMessage('高亮已删除。')
    } catch (highlightError) {
      setPanelMessage(highlightError instanceof ApiError ? highlightError.message : '删除高亮失败。')
    }
  }

  const handleDeleteActiveHighlight = async () => {
    if (!activeHighlight) {
      setPanelMessage('当前选区还没有对应的高亮。')
      return
    }

    await handleDeleteHighlight(activeHighlight.id)
  }

  const handleHighlightSwatchMouseDown = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault()
  }

  const handleResetProgress = async () => {
    if (!document) return

    try {
      setIsSavingProgress(true)
      const nextProgress = await api<DocumentProgress>(`/documents/${id}/progress`, {
        method: 'PUT',
        body: JSON.stringify({
          paragraph_position: 0,
          progress_percent: 0,
        }),
      })
      setProgress(nextProgress)
      setActiveParagraphIndex(0)
      paragraphRefs.current[0]?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      setPanelMessage('阅读进度已重置。')
    } catch (resetError) {
      setPanelMessage(resetError instanceof ApiError ? resetError.message : '重置阅读进度失败。')
    } finally {
      setIsSavingProgress(false)
    }
  }

  const displayProgressPercent = document ? effectiveProgressPercent : (progress?.progress_percent ?? 0)

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] px-6 py-8">
      <div className="mx-auto max-w-[1500px]">
        <header className="mb-6 flex flex-wrap items-center justify-between gap-6 text-lg text-[var(--ink-strong)]">
          <div className="flex flex-wrap items-center gap-8">
            <Link className="transition hover:text-[var(--accent)]" to="/">首页</Link>
            <Link className="transition hover:text-[var(--accent)]" to="/documents">返回文献库</Link>
            {document ? <div className="text-sm text-[var(--ink-soft)]">当前文献：{document.title}</div> : null}
          </div>
          <div className="flex items-center gap-3 rounded-full border border-[var(--line-muted)] bg-white/60 px-4 py-2">
            <div className="text-right">
              <p className="text-[11px] tracking-[0.16em] text-[var(--ink-soft)]">登录管理</p>
              <p className="text-sm text-[var(--ink-strong)]">{user?.username ?? '未登录'}</p>
            </div>
            <button
              onClick={logout}
              className="rounded-full border border-[var(--line-strong)] px-4 py-2 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]"
            >
              退出登录
            </button>
          </div>
        </header>

        <main className="border-frame bg-[rgba(244,248,244,0.6)] p-4 shadow-[var(--shadow-soft)]">
          <div className="grid min-h-[760px] grid-cols-[160px_minmax(0,1fr)_360px] gap-8 border border-[rgba(50,116,109,0.9)] p-4">
            {isLoading ? (
              <div className="col-span-3 flex items-center justify-center text-[var(--ink-soft)]">正在加载文献内容...</div>
            ) : error ? (
              <div className="col-span-3 rounded-[18px] border border-[rgba(180,90,90,0.22)] bg-[rgba(255,240,240,0.7)] px-4 py-3 text-sm text-[#8b3d3d]">{error}</div>
            ) : !document ? (
              <div className="col-span-3 flex items-center justify-center text-[var(--ink-soft)]">文献不存在。</div>
            ) : (
              <>
                <aside className="border border-[rgba(50,116,109,0.9)] bg-[rgba(233,241,235,0.72)] px-5 py-6">
                  <p className="font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">段落导航</p>
                  <div className="mt-4 text-sm text-[var(--ink-soft)]">当前定位：第 {activeParagraphIndex + 1} 段</div>
                  <div className="mt-8 space-y-4">
                    {document.paragraphs.map((paragraph, index) => (
                      <button key={paragraph.id} onClick={() => handleSelectParagraph(index)} className={`block text-left text-base transition ${index === activeParagraphIndex ? 'text-[var(--accent)]' : 'text-[var(--ink-main)]'}`}>第 {index + 1} 段</button>
                    ))}
                  </div>
                </aside>

                <section className="border border-[rgba(50,116,109,0.9)] bg-[rgba(245,248,242,0.84)] px-8 py-8">
                  <div className="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--line-muted)] pb-4">
                    <div>
                      <p className="font-[var(--font-ui)] text-sm tracking-[0.18em] text-[var(--ink-soft)]">原文阅读</p>
                      <h1 className="mt-2 font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">{document.title}</h1>
                      <p className="mt-2 text-sm text-[var(--ink-soft)]">{document.original_filename} · {document.source_type === 'markdown' ? 'Markdown' : 'TXT'}</p>
                    </div>
                    <div className="min-w-[280px]">
                      <div className="flex flex-wrap items-center justify-between gap-3 text-sm text-[var(--ink-soft)]">
                        <div className="flex items-center gap-3">
                          <span>阅读进度</span>
                          <button onClick={handleResetProgress} className="rounded-full border border-[var(--line-strong)] bg-white/75 px-3 py-1 text-xs text-[var(--ink-main)] transition hover:border-[var(--accent)] hover:text-[var(--accent)]">
                            重置进度
                          </button>
                        </div>
                        <span>{displayProgressPercent.toFixed(1)}%</span>
                      </div>
                      <div className="mt-2 h-2 overflow-hidden rounded-full bg-[rgba(16,79,85,0.12)]">
                        <div className="h-full rounded-full bg-[#2f7d73] transition-[width] duration-300 ease-out" style={{ width: displayProgressPercent <= 0 ? '0%' : `max(${displayProgressPercent}%, 10px)` }} />
                      </div>
                      <div className="mt-2 text-right text-xs text-[var(--ink-soft)]">{isSavingProgress ? '正在同步进度...' : `已记录到第 ${activeParagraphIndex + 1} 段`}</div>
                    </div>
                  </div>

                  <div className="mt-4 rounded-[16px] border border-dashed border-[var(--line-muted)] bg-white/40 px-4 py-3 text-sm text-[var(--ink-soft)]">高亮使用方式：在下方正文中用鼠标左键按住拖动，选中同一段落内的一小段文字。选中后会在文字附近直接弹出操作浮层。</div>

                  <div ref={contentRef} className="relative mt-8 space-y-5" onMouseUp={handleTextSelection}>
                    {selection && selectionPopover ? (
                      <div ref={selectionToolsRef} className="absolute z-20 w-[320px] rounded-[18px] border border-[var(--line-muted)] bg-[rgba(255,255,255,0.98)] p-4 shadow-[var(--shadow-soft)]" style={{ left: Math.max(0, selectionPopover.x), top: Math.max(0, selectionPopover.y) }}>
                        <p className="text-xs text-[var(--ink-soft)]">当前选中文本</p>
                        <p className="mt-2 line-clamp-2 text-sm text-[var(--ink-main)]">{selection.text}</p>
                        <div className="mt-3 grid grid-cols-3 gap-2">
                          {highlightColors.map((color) => (
                            <button key={color} onMouseDown={handleHighlightSwatchMouseDown} onClick={() => setHighlightColor(color)} className={`flex items-center justify-center gap-2 rounded-full border px-3 py-2 text-xs ${highlightColor === color ? 'border-[var(--ink-strong)] bg-[rgba(16,79,85,0.08)] text-[var(--ink-strong)]' : 'border-[var(--line-strong)] text-[var(--ink-main)]'}`}>
                              <span className={`h-2.5 w-2.5 rounded-full ${colorDotClassMap[color]}`} />
                              <span>{colorLabelMap[color]}</span>
                            </button>
                          ))}
                        </div>
                        <div className="mt-4 grid gap-2">
                          <button onMouseDown={handleHighlightSwatchMouseDown} onClick={handleCreateHighlight} className="flex w-full items-center justify-center rounded-full border border-[#104f55] bg-[#104f55] px-4 py-2.5 text-sm font-medium text-white shadow-[0_8px_18px_rgba(16,79,85,0.18)]">
                            {activeHighlight ? '更新高亮颜色' : '添加高亮'}
                          </button>
                          <div className="grid grid-cols-2 gap-2">
                            {activeHighlight ? <button onMouseDown={handleHighlightSwatchMouseDown} onClick={handleDeleteActiveHighlight} className="rounded-full border border-[#8b3d3d] px-3 py-2 text-sm text-[#8b3d3d]">清除高亮</button> : <div />}
                            <button onClick={clearSelection} className="rounded-full border border-[var(--line-strong)] px-3 py-2 text-sm">取消</button>
                          </div>
                        </div>
                      </div>
                    ) : null}

                    {document.paragraphs.map((paragraph, index) => {
                      const segments = applyHighlights(paragraph.content, paragraph.id, highlights)
                      return (
                        <article ref={(node) => { paragraphRefs.current[index] = node }} key={paragraph.id} data-paragraph-id={paragraph.id} onClick={() => handleSelectParagraph(index)} className={`rounded-[20px] border px-6 py-5 text-lg leading-9 transition select-text ${index === activeParagraphIndex ? 'border-[var(--accent)] bg-[rgba(255,255,255,0.82)]' : 'border-[var(--line-muted)] bg-[rgba(255,255,255,0.56)]'}`}>
                          {segments.map((segment, segmentIndex) => (<span key={`${paragraph.id}-${segmentIndex}`} className={segment.color ? `${colorClassMap[segment.color]} rounded px-0.5` : ''}>{segment.text}</span>))}
                        </article>
                      )
                    })}
                  </div>
                </section>

                <aside className="border border-[rgba(50,116,109,0.9)] bg-[rgba(233,241,235,0.72)] px-6 py-6">
                  <p className="font-[var(--font-reading)] text-2xl text-[var(--ink-strong)]">AI 与学习面板</p>

                  <div className="mt-6 rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.52)] p-4">
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-sm text-[var(--ink-soft)]">当前选中文本</p>
                      {selection ? <button onClick={clearSelection} className="text-xs text-[var(--ink-soft)] hover:text-[var(--accent)]">清除选择</button> : null}
                    </div>
                    {selection ? (
                      <>
                        <p className="mt-2 text-sm leading-7 text-[var(--ink-main)]">{selection.text}</p>
                        <p className="mt-2 text-xs text-[var(--ink-soft)]">第 {selection.paragraph.position + 1} 段 · 偏移 {selection.startOffset} - {selection.endOffset}</p>
                        <div className="mt-4 grid grid-cols-3 gap-2">
                          {highlightColors.map((color) => (
                            <button key={color} onMouseDown={handleHighlightSwatchMouseDown} onClick={() => setHighlightColor(color)} className={`flex items-center justify-center gap-2 rounded-full border px-3 py-2 text-xs ${highlightColor === color ? 'border-[var(--ink-strong)] bg-[rgba(16,79,85,0.08)] text-[var(--ink-strong)]' : 'border-[var(--line-strong)] text-[var(--ink-main)]'}`}>
                              <span className={`h-2.5 w-2.5 rounded-full ${colorDotClassMap[color]}`} />
                              <span>{colorLabelMap[color]}</span>
                            </button>
                          ))}
                        </div>
                        <div className="mt-4 flex gap-2">
                          <button onMouseDown={handleHighlightSwatchMouseDown} onClick={handleCreateHighlight} className="flex flex-1 items-center justify-center rounded-full border border-[#104f55] bg-[#104f55] px-4 py-2 text-sm font-medium text-white shadow-[0_8px_18px_rgba(16,79,85,0.12)] transition hover:bg-[#0d4348]">{activeHighlight ? '更新高亮颜色' : '添加高亮'}</button>
                          {activeHighlight ? <button onMouseDown={handleHighlightSwatchMouseDown} onClick={handleDeleteActiveHighlight} className="rounded-full border border-[#8b3d3d] px-4 py-2 text-sm text-[#8b3d3d]">清除</button> : null}
                        </div>
                      </>
                    ) : <p className="mt-2 text-sm leading-7 text-[var(--ink-soft)]">还没有选中文本。请到左侧正文区域里，用鼠标左键按住拖动选择一小段文字。</p>}
                  </div>

                  <div className="mt-6 space-y-4">
                    <div className="rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-4"><p className="text-sm text-[var(--ink-soft)]">划词翻译</p><p className="mt-2 text-sm leading-7 text-[var(--ink-main)]">{selection ? buildDemoTranslation(selection.text) : '选中文本后，这里会显示翻译结果。当前为前端演示版。'}</p></div>
                    <div className="rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-4"><p className="text-sm text-[var(--ink-soft)]">长难句解析</p><div className="mt-2 space-y-2 text-sm leading-7 text-[var(--ink-main)]">{(selection ? buildDemoAnalysis(selection.text) : ['点击段落并划词后，可在这里展示 AI 解析。当前先保留交互位置。']).map((item) => (<p key={item}>{item}</p>))}</div></div>
                  </div>

                  <div className="mt-6 rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-4"><div className="flex items-center justify-between"><p className="text-sm text-[var(--ink-soft)]">文献标签</p><button onClick={handleSaveTags} className="rounded-full border border-[var(--line-strong)] px-3 py-1 text-xs transition hover:border-[var(--accent)] hover:text-[var(--accent)]">保存标签</button></div><div className="mt-3 flex flex-wrap gap-2">{documentTags.length === 0 ? <span className="text-sm text-[var(--ink-soft)]">当前还没有绑定标签。</span> : documentTags.map((tag) => (<span key={tag.id} className="rounded-full bg-[rgba(50,116,109,0.12)] px-3 py-1 text-sm text-[var(--ink-strong)]">{tag.name}</span>))}</div><div className="mt-4 space-y-2">{allTags.map((tag) => (<label key={tag.id} className="flex items-center gap-2 text-sm text-[var(--ink-main)]"><input type="checkbox" checked={selectedTagIds.includes(tag.id)} onChange={(event) => setSelectedTagIds((current) => event.target.checked ? [...current, tag.id] : current.filter((item) => item !== tag.id))} />{tag.name}</label>))}</div><div className="mt-4 flex gap-2"><input value={newTagName} onChange={(event) => setNewTagName(event.target.value)} placeholder="新建标签" className="min-w-0 flex-1 rounded-[14px] border border-[var(--line-muted)] bg-white/70 px-3 py-2 text-sm outline-none" /><button onClick={handleCreateTag} className="rounded-full border border-[var(--line-strong)] px-4 py-2 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]">新建</button></div></div>

                  <div className="mt-6 rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-4"><p className="text-sm text-[var(--ink-soft)]">阅读笔记</p><select value={noteParagraphId} onChange={(event) => setNoteParagraphId(event.target.value)} className="mt-3 w-full rounded-[14px] border border-[var(--line-muted)] bg-white/70 px-3 py-2 text-sm outline-none"><option value="document">整篇文献</option>{document.paragraphs.map((paragraph, index) => (<option key={paragraph.id} value={paragraph.id}>第 {index + 1} 段</option>))}</select><textarea value={noteContent} onChange={(event) => setNoteContent(event.target.value)} rows={4} placeholder="记录这一段的理解、问题或待查资料" className="mt-3 w-full rounded-[14px] border border-[var(--line-muted)] bg-white/70 px-3 py-3 text-sm outline-none" /><button onClick={handleCreateNote} className="mt-3 rounded-full border border-[var(--line-strong)] px-4 py-2 text-sm transition hover:border-[var(--accent)] hover:text-[var(--accent)]">添加笔记</button><div className="mt-4 space-y-3">{notes.map((note) => (<div key={note.id} className="rounded-[16px] border border-[var(--line-muted)] bg-white/60 p-3"><div className="flex items-center justify-between gap-3"><div className="text-xs text-[var(--ink-soft)]">{note.paragraph_id ? '段落笔记' : '文献笔记'} · {formatDate(note.updated_at)}</div><button onClick={() => handleDeleteNote(note.id)} className="text-xs text-[#8b3d3d]">删除</button></div><p className="mt-2 text-sm leading-7 text-[var(--ink-main)]">{note.content}</p></div>))}</div></div>

                  <div className="mt-6 rounded-sm border border-[var(--line-muted)] bg-[rgba(255,255,255,0.44)] p-4"><p className="text-sm text-[var(--ink-soft)]">重点高亮记录</p><div className="mt-4 space-y-3">{highlights.length === 0 ? <p className="text-sm text-[var(--ink-soft)]">还没有高亮内容。</p> : highlights.map((item) => { const paragraph = document.paragraphs.find((entry) => entry.id === item.paragraph_id); const preview = paragraph ? sliceChars(paragraph.content, item.start_offset, item.end_offset) : ''; return (<div key={item.id} className="rounded-[16px] border border-[var(--line-muted)] bg-white/60 p-3"><div className="flex items-center justify-between gap-3"><div className="text-xs text-[var(--ink-soft)]">第 {(paragraph?.position ?? 0) + 1} 段 · {colorLabelMap[item.color]}</div><button onClick={() => handleDeleteHighlight(item.id)} className="text-xs text-[#8b3d3d]">删除</button></div><p className="mt-2 text-sm text-[var(--ink-main)]">{preview || '高亮片段预览不可用'}</p></div>)})}</div></div>

                  {panelMessage ? <div className="mt-6 rounded-[16px] border border-[var(--line-muted)] bg-white/60 px-4 py-3 text-sm text-[var(--ink-main)]">{panelMessage}</div> : null}
                </aside>
              </>
            )}
          </div>
        </main>
      </div>
    </div>
  )
}



