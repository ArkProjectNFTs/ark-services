ALTER TABLE orderbook_token
    ADD COLUMN order_hash TEXT NOT NULL DEFAULT '';

ALTER TABLE orderbook_token_offers
    ADD COLUMN order_hash TEXT NOT NULL DEFAULT '';

ALTER TABLE orderbook_token
    ADD COLUMN currency_address TEXT NOT NULL DEFAULT '',
    ADD COLUMN currency_chain_id TEXT NOT NULL DEFAULT '';

ALTER TABLE orderbook_token_offers
    ADD COLUMN currency_address TEXT NOT NULL DEFAULT '',
    ADD COLUMN currency_chain_id TEXT NOT NULL DEFAULT '';
