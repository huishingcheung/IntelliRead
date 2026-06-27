import { api } from './client'
import type {
  CreateVocabularyInput,
  MasteryStatus,
  ReviewAnswer,
  ReviewResult,
  UpdateVocabularyInput,
  VocabularyCard,
  VocabularyListResponse,
} from '../types'

export type VocabularyListQuery = {
  page?: number
  pageSize?: number
  sort?: 'created_at' | 'next_review_at' | 'term'
  order?: 'asc' | 'desc'
  masteryStatus?: MasteryStatus | 'all'
  documentId?: string
}

function withQuery(path: string, entries: Array<[string, string | number | undefined]>) {
  const search = new URLSearchParams()

  entries.forEach(([key, value]) => {
    if (value !== undefined && value !== '') {
      search.set(key, String(value))
    }
  })

  const query = search.toString()
  return query ? `${path}?${query}` : path
}

export function listVocabulary(query: VocabularyListQuery = {}) {
  return api<VocabularyListResponse>(
    withQuery('/vocabulary', [
      ['page', query.page],
      ['page_size', query.pageSize],
      ['sort', query.sort],
      ['order', query.order],
      ['mastery_status', query.masteryStatus === 'all' ? undefined : query.masteryStatus],
      ['document_id', query.documentId],
    ]),
  )
}

export function createVocabulary(input: CreateVocabularyInput) {
  return api<VocabularyCard>('/vocabulary', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export function updateVocabulary(id: string, input: UpdateVocabularyInput) {
  return api<VocabularyCard>(`/vocabulary/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(input),
  })
}

export function deleteVocabulary(id: string) {
  return api<void>(`/vocabulary/${id}`, { method: 'DELETE' })
}

export function getReviewQueue(limit = 20, documentId?: string) {
  return api<VocabularyCard[]>(
    withQuery('/review/queue', [
      ['limit', limit],
      ['document_id', documentId],
    ]),
  )
}

export function submitReviewAnswer(vocabularyId: string, answerResult: ReviewResult) {
  return api<ReviewAnswer>('/review/answer', {
    method: 'POST',
    body: JSON.stringify({
      vocabulary_id: vocabularyId,
      answer_result: answerResult,
    }),
  })
}
