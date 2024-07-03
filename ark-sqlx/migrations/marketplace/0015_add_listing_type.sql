ALTER TABLE token
    ADD COLUMN  topbid_start_date BIGINT NULL,
    ADD COLUMN  topbid_end_date BIGINT NULL,
    ADD COLUMN topbid_currency_address TEXT NULL;
