ALTER TABLE token
    ADD COLUMN  top_bid_start_date BIGINT NULL,
    ADD COLUMN  top_bid_end_date BIGINT NULL,
    ADD COLUMN top_bid_currency_address TEXT NULL;
