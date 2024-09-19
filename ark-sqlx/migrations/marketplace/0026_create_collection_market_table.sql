CREATE TABLE contract_marketdata (
           contract_address VARCHAR(255),
           chain_id TEXT NOT NULL,
           floor_percentage NUMERIC,
           volume BIGINT,
           number_of_sales INTEGER,
           timerange VARCHAR(10),
           CHECK (timerange IN ('10m', '1h', '6h', '1d', '7d', '30d')),
           PRIMARY KEY (contract_address, chain_id, timerange)
);

ALTER TABLE contract ADD COLUMN calculate_marketdata_timestamp BIGINT NULL;
