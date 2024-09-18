CREATE TABLE contract_marketdata (
           contract_address VARCHAR(255),
           chain_id INTEGER,
           floor_percentage NUMERIC,
           volume NUMERIC,
           number_of_sales INTEGER,
           timerange VARCHAR(10),
           CHECK (timerange IN ('10m', '1h', '6h', '1d', '7d', '30d'))
);

ALTER TABLE contract ADD COLUMN calculate_marketdata_timestamp BIGINT NULL;
