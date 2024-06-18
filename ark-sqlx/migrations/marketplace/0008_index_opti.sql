DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM   pg_class c
        JOIN   pg_namespace n ON n.oid = c.relnamespace
        WHERE  c.relname = 'idx_token_contract_chain_listing_start_amount_token_id'
        AND    n.nspname = 'public'
    ) THEN
DROP INDEX public.idx_token_contract_chain_listing_start_amount_token_id;
END IF;
END $$;

CREATE INDEX idx_token_contract_chain_listing_start_amount_token_id ON token (contract_address, chain_id, listing_start_amount, (CAST(token_id AS NUMERIC)));
