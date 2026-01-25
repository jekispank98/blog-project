DROP TABLE IF EXISTS users CASCADE;

CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       username VARCHAR NOT NULL UNIQUE,
                       email VARCHAR NOT NULL UNIQUE,
                       password_hash VARCHAR NOT NULL,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_email ON users (email);