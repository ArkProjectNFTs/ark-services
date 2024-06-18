DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM   pg_class c
        JOIN   pg_namespace n ON n.oid = c.relnamespace
        WHERE  c.relname = 'idx_token_id_numeric'
        AND    n.nspname = 'public'
    ) THEN
DROP INDEX public.idx_token_id_numeric;
END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM   pg_class c
        JOIN   pg_namespace n ON n.oid = c.relnamespace
        WHERE  c.relname = 'idx_listing_dates'
        AND    n.nspname = 'public'
    ) THEN
DROP INDEX public.idx_listing_dates;
END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM   pg_class c
        JOIN   pg_namespace n ON n.oid = c.relnamespace
        WHERE  c.relname = 'idx_listing_start_amount'
        AND    n.nspname = 'public'
    ) THEN
DROP INDEX public.idx_listing_start_amount;
END IF;
END $$;

CREATE INDEX idx_token_id_numeric ON token ((CAST(token_id AS NUMERIC)));
CREATE INDEX idx_listing_dates ON token (listing_start_date, listing_end_date);
CREATE INDEX idx_listing_start_amount ON token (listing_start_amount);
CREATE INDEX idx_token_contract_chain_listing_start_amount_token_id ON token (contract_address, chain_id, listing_start_amount, (CAST(token_id AS NUMERIC)));
