CREATE INDEX idx_token_id_numeric ON token ((CAST(token_id AS NUMERIC)));
CREATE INDEX idx_listing_dates ON token (listing_start_date, listing_end_date);
CREATE INDEX idx_listing_start_amount ON token (listing_start_amount);
CREATE INDEX idx_token_contract_chain_listing_start_amount_token_id ON token (contract_address, chain_id, listing_start_amount, (CAST(token_id AS NUMERIC)));
