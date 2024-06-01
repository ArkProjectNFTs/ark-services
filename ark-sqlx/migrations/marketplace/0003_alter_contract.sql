ALTER TABLE token REMOVE COLUMN deployed_timestamp BIGINT;
ALTER TABLE contract ADD COLUMN deployed_timestamp BIGINT;