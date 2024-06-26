ALTER TABLE contract ADD COLUMN top_bid_order_hash TEXT;

CREATE INDEX idx_listed_amount_id ON token(
    (CASE WHEN is_listed = true THEN 1 ELSE 2 END),
    listing_start_amount,
    CAST(token_id AS NUMERIC)
);
