ALTER TABLE token
ALTER COLUMN top_bid_amount TYPE numeric USING top_bid_amount::numeric;

ALTER TABLE contract
ALTER COLUMN floor_price TYPE numeric USING floor_price::numeric,
ALTER COLUMN top_bid TYPE numeric USING top_bid::numeric;
