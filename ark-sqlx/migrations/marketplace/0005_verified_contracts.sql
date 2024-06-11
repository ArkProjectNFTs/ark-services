ALTER TABLE contract 
ADD COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX idx_is_verified ON contract(is_verified);
