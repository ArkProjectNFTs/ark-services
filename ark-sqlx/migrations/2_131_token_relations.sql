-- SQL migration for arkchain orderbook.
--
CREATE TABLE orderbook_token (
       token_chain_id TEXT NOT NULL,
       token_address TEXT NOT NULL,
       token_id TEXT NOT NULL,
       listed_timestamp BIGINT NOT NULL,
       updated_timestamp BIGINT NOT NULL,
       status TEXT NOT NULL,
       current_owner TEXT NOT NULL,
       current_amount TEXT NOT NULL,

       quantity TEXT NOT NULL,
       start_amount TEXT NOT NULL,
       end_amount TEXT NOT NULL,
       start_date BIGINT NOT NULL,
       end_date BIGINT NOT NULL,
       broker_id TEXT NOT NULL,

       PRIMARY KEY (token_id)
);

CREATE TABLE orderbook_token_history (
       history_id SERIAL PRIMARY KEY,
       token_id TEXT NOT NULL,
       event_type TEXT NOT NULL,
       event_timestamp BIGINT NOT NULL,
       previous_owner TEXT,       -- NULL if new listing
       new_owner TEXT,            -- NULL if not transfert
       amount TEXT,
       FOREIGN KEY (token_id) REFERENCES orderbook_token(token_id)
);

CREATE TABLE orderbook_token_offers (
      offer_id SERIAL PRIMARY KEY,
      token_id TEXT NOT NULL,
      offer_maker TEXT NOT NULL,
      offer_amount TEXT NOT NULL,
      offer_quantity TEXT NOT NULL,
      offer_timestamp BIGINT NOT NULL,
      offer_status TEXT NOT NULL,
      offer_expiry BIGINT,
      FOREIGN KEY (token_id) REFERENCES orderbook_token(token_id)
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
