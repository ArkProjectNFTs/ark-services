CREATE INDEX idx_token_is_listed ON token (is_listed);

CREATE INDEX idx_token_order ON token (is_listed, contract_address, chain_id);
