CREATE TABLE vocabulary_cards (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    paragraph_id TEXT,
    term TEXT NOT NULL COLLATE NOCASE,
    pronunciation TEXT,
    definition TEXT NOT NULL,
    example_sentence TEXT,
    source_text TEXT,
    mastery_status TEXT NOT NULL DEFAULT 'new'
        CHECK (mastery_status IN ('new', 'learning', 'familiar', 'mastered')),
    next_review_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (paragraph_id) REFERENCES document_paragraphs(id) ON DELETE SET NULL,
    UNIQUE(user_id, document_id, term)
);

CREATE INDEX idx_vocabulary_user_document_created
    ON vocabulary_cards(user_id, document_id, created_at DESC);

CREATE INDEX idx_vocabulary_user_review
    ON vocabulary_cards(user_id, next_review_at, mastery_status);

CREATE TABLE review_answers (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    vocabulary_id TEXT NOT NULL,
    answer_result TEXT NOT NULL
        CHECK (answer_result IN ('wrong', 'hard', 'good', 'easy')),
    reviewed_at TEXT NOT NULL,
    next_review_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (vocabulary_id) REFERENCES vocabulary_cards(id) ON DELETE CASCADE
);

CREATE INDEX idx_review_answers_user_reviewed
    ON review_answers(user_id, reviewed_at DESC);

CREATE INDEX idx_review_answers_vocabulary_reviewed
    ON review_answers(vocabulary_id, reviewed_at DESC);