ALTER TABLE orderbook_token
    ADD COLUMN status TEXT NOT NULL DEFAULT 'PLACED';

ALTER TABLE orderbook_token_offers
    ADD COLUMN start_date BIGINT DEFAULT 0,
    ADD COLUMN end_date BIGINT DEFAULT 0 ;
