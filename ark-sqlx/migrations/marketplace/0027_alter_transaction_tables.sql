BEGIN;
ALTER TABLE transaction_info ADD COLUMN sub_event_id VARCHAR(78);

-- Mise à jour par lots
DO $$
DECLARE
    batch_size INT := 100000;
    total_rows INT;
    processed_rows INT := 0;
BEGIN
    SELECT COUNT(*) INTO total_rows FROM transaction_info;
    WHILE processed_rows < total_rows LOOP
        UPDATE transaction_info
        SET sub_event_id = concat(event_id, '_0')
        WHERE ctid IN (
            SELECT ctid
            FROM transaction_info
            WHERE sub_event_id IS NULL
            LIMIT batch_size
        );
        processed_rows := processed_rows + batch_size;
        COMMIT;
        RAISE NOTICE 'Processed % rows out of %', processed_rows, total_rows;
    END LOOP;
END $$;

ALTER TABLE transaction_info ALTER COLUMN sub_event_id SET NOT NULL;
ALTER TABLE transaction_info DROP CONSTRAINT transaction_info_tx_hash_event_id_key;
ALTER TABLE transaction_info ADD CONSTRAINT transaction_info_tx_hash_event_id_sub_event_id_key UNIQUE(tx_hash, event_id, sub_event_id);
COMMIT;