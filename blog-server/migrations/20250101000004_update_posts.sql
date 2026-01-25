DROP TABLE IF EXISTS posts CASCADE;
CREATE TABLE posts (
                       id UUID PRIMARY KEY,
                       title VARCHAR NOT NULL,
                       content TEXT NOT NULL,
                       author_id UUID NOT NULL,
                       created_at BIGINT NOT NULL,
                       updated_at BIGINT NOT NULL,
                       CONSTRAINT fk_author FOREIGN KEY (author_id) REFERENCES users(id)
);