ALTER TABLE orderbook_token_history
ADD COLUMN end_amount TEXT,
ADD COLUMN start_date BIGINT NULL,
ADD COLUMN end_date BIGINT NULL;
