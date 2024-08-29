DO $$
BEGIN
    -- Check and alter column volume_7d_eth if not already numeric
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'contract'
        AND column_name = 'volume_7d_eth'
        AND data_type != 'numeric'
    ) THEN
        ALTER TABLE contract
        ALTER COLUMN volume_7d_eth TYPE numeric;
    END IF;

    -- Check and alter column marketcap if not already numeric
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'contract'
        AND column_name = 'marketcap'
        AND data_type != 'numeric'
    ) THEN
        ALTER TABLE contract
        ALTER COLUMN marketcap TYPE numeric;
    END IF;

    -- Check and drop index idx_token_is_listed if exists
    IF EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_token_is_listed') THEN
        DROP INDEX idx_token_is_listed;
    END IF;

    -- Check and drop index idx_token_order if exists
    IF EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_token_order') THEN
        DROP INDEX idx_token_order;
    END IF;

    -- Check and drop column is_listed if exists
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'token'
        AND column_name = 'is_listed'
    ) THEN
        ALTER TABLE token
        DROP COLUMN is_listed;
    END IF;
END $$;