ALTER TABLE orders
RENAME COLUMN token_id TO token_id_hex;

ALTER TABLE orders
ADD COLUMN token_id TEXT;

ALTER TABLE active_orders
RENAME COLUMN token_id TO token_id_hex;

ALTER TABLE active_orders
ADD COLUMN token_id TEXT;
