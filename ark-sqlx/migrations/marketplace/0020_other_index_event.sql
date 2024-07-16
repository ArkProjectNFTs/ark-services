CREATE INDEX idx_token_event_chain_id ON token_event(chain_id);
CREATE INDEX idx_token_event_from_to_address ON token_event(from_address, to_address);
CREATE INDEX idx_token_event_order_hash ON token_event(order_hash);
CREATE INDEX idx_token_event_block_timestamp ON token_event(block_timestamp);
