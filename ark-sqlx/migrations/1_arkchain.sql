-- SQL migration for arkchain orderbook.
--
CREATE TABLE orderbook_order (
       block_id BIGINT NOT NULL,

       order_hash TEXT NOT NULL,
       order_version TEXT NOT NULL,
       order_type TEXT NOT NULL,
       cancelled_order_hash TEXT DEFAULT NULL,

       -- Order V1.
       route TEXT NOT NULL,
       currency_address TEXT NOT NULL,
       currency_chain_id TEXT NOT NULL,
       salt TEXT NOT NULL,
       offerer TEXT NOT NULL,
       token_chain_id TEXT NOT NULL,
       token_address TEXT NOT NULL,
       token_id TEXT NOT NULL,
       quantity TEXT NOT NULL,
       start_amount TEXT NOT NULL,
       end_amount TEXT NOT NULL,
       start_date BIGINT NOT NULL,
       end_date BIGINT NOT NULL,
       broker_id TEXT NOT NULL,

       PRIMARY KEY (order_hash)
);
