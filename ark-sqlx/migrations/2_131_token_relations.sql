-- SQL migration for arkchain orderbook.
--
CREATE TABLE orderbook_token (
       token_chain_id TEXT NOT NULL,
       token_address TEXT NOT NULL,
       token_id TEXT NOT NULL,
       listed_timestamp BIGINT NOT NULL,
       updated_timestamp BIGINT NOT NULL,
       current_owner TEXT NOT NULL,
       current_price TEXT NULL,

       quantity TEXT NULL,
       start_amount TEXT NULL,
       end_amount TEXT NULL,
       start_date BIGINT NULL,
       end_date BIGINT NULL,
       broker_id TEXT NULL,

       PRIMARY KEY (token_id, token_address)
);

CREATE TABLE orderbook_token_history (
       history_id SERIAL PRIMARY KEY,
       token_id TEXT NOT NULL,
       token_address TEXT NOT NULL,
       event_type TEXT NOT NULL,
       event_timestamp BIGINT NOT NULL,
       order_status TEXT NOT NULL,
       previous_owner TEXT NULL,       -- NULL if new listing
       new_owner TEXT NULL,            -- NULL if not transfert
       amount TEXT NULL,
       canceled_reason TEXT NULL, -- NULL if not cancelled
       FOREIGN KEY (token_id, token_address) REFERENCES orderbook_token(token_id, token_address)
);

CREATE TABLE orderbook_token_offers (
      offer_id SERIAL PRIMARY KEY,
      token_id TEXT NOT NULL,
      token_address TEXT NOT NULL,
      offer_maker TEXT NOT NULL,
      offer_amount TEXT NOT NULL,
      offer_quantity TEXT NOT NULL,
      offer_timestamp BIGINT NOT NULL,
      FOREIGN KEY (token_id, token_address) REFERENCES orderbook_token(token_id, token_address)
);

-- rename orderbook_order_placed to orderbook_order_created
ALTER TABLE orderbook_order_placed
    RENAME TO orderbook_order_created;
