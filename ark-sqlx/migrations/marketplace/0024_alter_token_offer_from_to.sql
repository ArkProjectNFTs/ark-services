DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='token_offer' AND column_name='to_address') THEN
ALTER TABLE token_offer ADD COLUMN to_address TEXT NULL;
END IF;
END $$;
