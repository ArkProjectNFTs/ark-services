ALTER TABLE token DROP COLUMN deployed_timestamp;
ALTER TABLE contract ADD COLUMN deployed_timestamp BIGINT;
