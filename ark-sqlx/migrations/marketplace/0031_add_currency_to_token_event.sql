ALTER TABLE token_event 
ADD COLUMN currency_address TEXT,
ON DELETE CASCADE;
