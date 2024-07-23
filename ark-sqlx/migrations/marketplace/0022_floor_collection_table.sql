CREATE TABLE floor_collection (
    contract_address VARCHAR(66) NOT NULL,
    chain_id TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    floor NUMERIC NOT NULL,
    PRIMARY KEY (contract_address, chain_id, timestamp));

ALTER TABLE contract ADD COLUMN floor_7d_percentage NUMERIC;
