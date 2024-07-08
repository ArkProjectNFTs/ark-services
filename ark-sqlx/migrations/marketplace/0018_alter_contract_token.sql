ALTER TABLE contract
ALTER COLUMN volume_7d_eth TYPE numeric,
ALTER COLUMN marketcap TYPE numeric;


ALTER TABLE token
    DROP COLUMN is_listed;
