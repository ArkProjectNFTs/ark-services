ALTER TABLE token 
    ADD COLUMN metadata_status TEXT DEFAULT('TO_REFRESH') NOT NULL;

UPDATE token 
    SET metadata_status = CASE 
        WHEN metadata_ok THEN 'OK' 
        ELSE 'TO_REFRESH' 
    END;

ALTER TABLE token 
DROP COLUMN metadata_ok;


ALTER TABLE token ADD COLUMN metadata_temp JSONB;
UPDATE token SET metadata_temp = metadata::JSONB;
ALTER TABLE token DROP COLUMN metadata;
ALTER TABLE token RENAME COLUMN metadata_temp TO metadata;
ALTER TABLE token ADD COLUMN raw_metadata TEXT NULL;
ALTER TABLE token ADD COLUMN metadata_updated_at BIGINT NULL;