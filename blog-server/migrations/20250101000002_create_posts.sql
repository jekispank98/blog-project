CREATE TABLE IF NOT EXISTS posts (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR,
    content TEXT,
    author_id BIGINT, FOREIGN KEY на users.id,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
)
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
CREATE INDEX IF NOT EXISTS idx_created_at ON users (created_at);
CREATE INDEX IF NOT EXISTS idx_author_id ON users (author_id);