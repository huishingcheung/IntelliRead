import { Navigate, Route, Routes } from 'react-router-dom'
import { ProtectedRoute } from './router/ProtectedRoute'
import { HomePage } from './pages/HomePage'
import { LoginPage } from './pages/LoginPage'
import { RegisterPage } from './pages/RegisterPage'
import { DocumentsPage } from './pages/DocumentsPage'
import { DocumentReaderPage } from './pages/DocumentReaderPage'
import { DocumentNotesTagsPage } from './pages/DocumentNotesTagsPage'
import { VocabularyPage } from './pages/VocabularyPage'
import { ReviewPage } from './pages/ReviewPage'

function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />

      <Route element={<ProtectedRoute />}>
        <Route path="/" element={<HomePage />} />
        <Route path="/documents" element={<DocumentsPage />} />
        <Route path="/documents/:id" element={<DocumentReaderPage />} />
        <Route path="/documents/:id/notes-tags" element={<DocumentNotesTagsPage />} />
        <Route path="/vocabulary" element={<VocabularyPage />} />
        <Route path="/review" element={<ReviewPage />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  )
}

export default App
