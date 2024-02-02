ALTER TABLE orderbook_token
    ADD COLUMN order_hash TEXT NOT NULL DEFAULT '';

ALTER TABLE orderbook_token_offers
    ADD COLUMN order_hash TEXT NOT NULL DEFAULT '';
