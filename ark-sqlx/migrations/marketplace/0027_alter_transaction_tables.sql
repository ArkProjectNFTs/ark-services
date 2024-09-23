DO $$
DECLARE
    batch_size INT := 100000;
    total_rows INT;
    processed_rows INT := 0;
    error_count INT := 0;
    max_errors INT := 5;
    column_exists BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'transaction_info' AND column_name = 'sub_event_id'
    ) INTO column_exists;

    IF NOT column_exists THEN
        ALTER TABLE transaction_info ADD COLUMN sub_event_id VARCHAR(78);
        RAISE NOTICE 'Colonne sub_event_id ajoutée à la table transaction_info';
    ELSE
        RAISE NOTICE 'La colonne sub_event_id existe déjà dans la table transaction_info';
    END IF;

    SELECT COUNT(*) INTO total_rows FROM transaction_info WHERE sub_event_id IS NULL;
    RAISE NOTICE 'Nombre total de lignes à mettre à jour : %', total_rows;

    WHILE processed_rows < total_rows LOOP
        BEGIN
            WITH updated_rows AS (
                SELECT ctid
                FROM transaction_info
                WHERE sub_event_id IS NULL
                LIMIT batch_size
                FOR UPDATE SKIP LOCKED
            )
            UPDATE transaction_info t
            SET sub_event_id = concat(event_id, '_0')
            FROM updated_rows
            WHERE t.ctid = updated_rows.ctid;

            GET DIAGNOSTICS processed_rows = ROW_COUNT;
            RAISE NOTICE 'Processed % rows out of %', processed_rows, total_rows;
            
            COMMIT;
        EXCEPTION WHEN OTHERS THEN
            ROLLBACK;
            error_count := error_count + 1;
            RAISE WARNING 'Error occurred: %. Retrying...', SQLERRM;
            IF error_count >= max_errors THEN
                RAISE EXCEPTION 'Max error count reached. Aborting.';
            END IF;
            PERFORM pg_sleep(5);
        END;
    END LOOP;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.table_constraints
        WHERE constraint_name = 'transaction_info_tx_hash_event_id_sub_event_id_key'
    ) THEN
        ALTER TABLE transaction_info ALTER COLUMN sub_event_id SET NOT NULL;
        ALTER TABLE transaction_info DROP CONSTRAINT IF EXISTS transaction_info_tx_hash_event_id_key;
        ALTER TABLE transaction_info ADD CONSTRAINT transaction_info_tx_hash_event_id_sub_event_id_key UNIQUE(tx_hash, event_id, sub_event_id);
        RAISE NOTICE 'Contraintes ajoutées à la table transaction_info';
    ELSE
        RAISE NOTICE 'Les contraintes existent déjà sur la table transaction_info';
    END IF;
END $$;