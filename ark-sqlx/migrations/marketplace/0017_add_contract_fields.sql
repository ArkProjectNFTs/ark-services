ALTER TABLE contract
    ADD COLUMN  owner_count BIGINT NULL,
    ADD COLUMN  token_count BIGINT NULL,
    ADD COLUMN  token_listed_count BIGINT NULL,
    ADD COLUMN  listed_percentage BIGINT NULL,
    ADD COLUMN  volume_7d_eth BIGINT NULL,
    ADD COLUMN  sales_7d BIGINT NULL,
    ADD COLUMN  total_volume BIGINT NULL,
    ADD COLUMN  total_sales BIGINT NULL,
    ADD COLUMN  marketcap BIGINT NULL;
