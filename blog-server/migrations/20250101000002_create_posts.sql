CREATE TABLE IF NOT EXISTS posts (
                                     id BIGSERIAL PRIMARY KEY,
                                     title VARCHAR NOT NULL,
                                     content TEXT NOT NULL,
                                     author_id BIGINT NOT NULL,
                                     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT fk_author
    FOREIGN KEY (author_id)
    REFERENCES users(id)
    ON DELETE CASCADE
    );