CREATE INDEX idx_token_contract_chain_listing_start_amount_token_id ON token (contract_address, chain_id, listing_start_amount, (CAST(token_id AS NUMERIC)));
