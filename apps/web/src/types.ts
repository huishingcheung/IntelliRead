export type ApiSuccessResponse<T> = {
  success: true
  data: T
}

export type ApiErrorResponse = {
  success: false
  error: {
    code: string
    message: string
  }
}

export type User = {
  id: string
  username: string
  email: string
  created_at: string
}

export type AuthPayload = {
  access_token: string
  token_type: string
  expires_in: number
  user: User
}

export type DocumentSummary = {
  id: string
  title: string
  source_type: string
  original_filename: string
  byte_size: number
  created_at: string
  updated_at: string
  archived_at: string | null
}

export type DocumentListItem = DocumentSummary

export type Paragraph = {
  id: string
  position: number
  content: string
}

export type DocumentDetail = DocumentSummary & {
  paragraphs: Paragraph[]
}

export type DocumentListResponse = {
  items: DocumentListItem[]
  limit: number
  offset: number
}

export type DocumentProgress = {
  document_id: string
  paragraph_position: number
  progress_percent: number
  updated_at: string
}

export type Tag = {
  id: string
  name: string
  created_at: string
}

export type Note = {
  id: string
  document_id: string
  paragraph_id: string | null
  content: string
  created_at: string
  updated_at: string
}

export type HighlightColor = 'yellow' | 'green' | 'blue' | 'pink' | 'purple'

export type Highlight = {
  id: string
  document_id: string
  paragraph_id: string
  start_offset: number
  end_offset: number
  color: HighlightColor
  created_at: string
  updated_at: string
}

export type LearningOverview = {
  active_documents: number
  archived_documents: number
  paragraphs: number
  tags: number
  notes: number
  highlights: number
  tracked_documents: number
  average_progress_percent: number
}
