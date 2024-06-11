CREATE INDEX idx_token_contract_address ON token(contract_address);
CREATE INDEX idx_token_chain_id ON token(chain_id);
CREATE INDEX idx_token_listing_start_date ON token(listing_start_date);
CREATE INDEX idx_token_listing_end_date ON token(listing_end_date);

CREATE INDEX idx_token_listing_start_amount ON token(listing_start_amount);

CREATE INDEX idx_token_listing_start_date_end_date ON token(listing_start_date, listing_end_date);
CREATE INDEX idx_token_listing_start_amount_order ON token(listing_start_amount);

CREATE INDEX idx_t1_contract_address ON token(contract_address);
CREATE INDEX idx_t1_listing_start_amount ON token(listing_start_amount);
