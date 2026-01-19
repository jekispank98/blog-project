CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR UNIQUE,
    email VARCHAR UNIQUE,
    password_hash VARCHAR,
    created_at TIMESTAMP WITH TIME ZONE
)
CREATE INDEX IF NOT EXISTS idx_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_email ON users (email);