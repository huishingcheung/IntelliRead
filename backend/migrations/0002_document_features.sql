ALTER TABLE documents ADD COLUMN archived_at TEXT;

CREATE INDEX idx_documents_user_archived_created
    ON documents(user_id, archived_at, created_at DESC);

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, name)
);

CREATE INDEX idx_tags_user_name ON tags(user_id, name);

CREATE TABLE document_tags (
    document_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (document_id, tag_id),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_document_tags_tag ON document_tags(tag_id, document_id);

CREATE TABLE notes (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    paragraph_id TEXT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (paragraph_id) REFERENCES document_paragraphs(id) ON DELETE CASCADE
);

CREATE INDEX idx_notes_user_document_created
    ON notes(user_id, document_id, created_at DESC);

CREATE TABLE highlights (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    paragraph_id TEXT NOT NULL,
    start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
    end_offset INTEGER NOT NULL CHECK (end_offset > start_offset),
    color TEXT NOT NULL DEFAULT 'yellow',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (paragraph_id) REFERENCES document_paragraphs(id) ON DELETE CASCADE
);

CREATE INDEX idx_highlights_user_document_created
    ON highlights(user_id, document_id, created_at DESC);
