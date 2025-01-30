ALTER TABLE token_event
RENAME COLUMN eth_amount TO amount_eth;

ALTER TABLE token_event 
ALTER COLUMN amount_eth TYPE numeric USING amount_eth::numeric;

