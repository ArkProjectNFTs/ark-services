-- SQL migration for arkchain orderbook.
--
CREATE TABLE orderbook_order_placed (
       block_id BIGINT NOT NULL,
       block_timestamp BIGINT NOT NULL,

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

CREATE TABLE orderbook_order_cancelled (
       block_id BIGINT NOT NULL,
       block_timestamp BIGINT NOT NULL,

       order_hash TEXT NOT NULL,
       reason TEXT NOT NULL,

       PRIMARY KEY (order_hash)
);

CREATE TABLE orderbook_order_fulfilled (
       block_id BIGINT NOT NULL,
       block_timestamp BIGINT NOT NULL,

       order_hash TEXT NOT NULL,
       fulfiller TEXT NOT NULL,
       related_order_hash TEXT DEFAULT NULL,

       PRIMARY KEY (order_hash)
);

CREATE TABLE orderbook_order_executed (
       block_id BIGINT NOT NULL,
       block_timestamp BIGINT NOT NULL,

       order_hash TEXT NOT NULL,

       PRIMARY KEY (order_hash)
);
