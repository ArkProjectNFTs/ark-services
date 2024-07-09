ALTER TABLE contract
ALTER COLUMN volume_7d_eth TYPE numeric,
ALTER COLUMN marketcap TYPE numeric;


drop index idx_token_is_listed;
drop index idx_token_order;


ALTER TABLE token
    DROP COLUMN is_listed;
