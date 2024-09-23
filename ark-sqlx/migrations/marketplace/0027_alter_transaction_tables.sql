BEGIN;
ALTER TABLE transaction_info add COLUMN sub_event_id VARCHAR(78);
UPDATE transaction_info set sub_event_id = concat(event_id, '_0');
ALTER TABLE transaction_info ALTER COLUMN sub_event_id SET NOT NULL;
ALTER TABLE transaction_info DROP CONSTRAINT transaction_info_tx_hash_event_id_key;
ALTER TABLE transaction_info ADD CONSTRAINT transaction_info_tx_hash_event_id_sub_event_id_key UNIQUE(tx_hash, event_id, sub_event_id);
COMMIT;