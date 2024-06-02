-- Add migration script here
CREATE INDEX idx_token_metadata_status ON token(metadata_status);