ALTER TABLE orderbook_token
    ALTER COLUMN current_owner DROP NOT NULL;

ALTER TABLE orderbook_token
    RENAME COLUMN current_price TO last_price;
