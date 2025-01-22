ALTER TABLE currency_mapping
RENAME TO currency;

ALTER TABLE currency
RENAME COLUMN currency_address TO contract_address;

ALTER TABLE currency
ADD COLUMN price_in_usd NUMERIC NOT NULL DEFAULT 0;

ALTER TABLE currency
ADD COLUMN price_in_eth NUMERIC NOT NULL DEFAULT 0;
