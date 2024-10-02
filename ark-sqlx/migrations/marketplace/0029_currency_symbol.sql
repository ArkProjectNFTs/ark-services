CREATE TABLE currency_mapping (
    currency_address TEXT,
    chain_id TEXT,
    symbol VARCHAR(78),
    decimals SMALLINT,
    PRIMARY KEY (currency_address, chain_id)
);