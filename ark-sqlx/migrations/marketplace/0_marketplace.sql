CREATE TABLE contract (
  contract_id SERIAL PRIMARY KEY,
  chain_id TEXT NOT NULL,
  updated_timestamp BIGINT NOT NULL,
  contract_address TEXT NOT NULL,
  floor_price TEXT,
  top_bid TEXT,
  contract_type TEXT NOT NULL CHECK (contract_type IN ('erc721', 'etc')),
  contract_name TEXT,
  contract_symbol TEXT,
  contract_image TEXT,
  metadata_ok BOOLEAN NOT NULL DEFAULT FALSE,
  is_spam BOOLEAN NOT NULL DEFAULT FALSE,
  is_nsfw BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE token (
   token_chain_id TEXT NOT NULL,
   contract_id INTEGER NOT NULL,
   token_id TEXT NOT NULL,
   current_owner TEXT,
   last_price TEXT NULL,
   currency_address TEXT NOT NULL DEFAULT '',
   currency_chain_id TEXT NOT NULL DEFAULT '',
   status TEXT NOT NULL DEFAULT 'PLACED',
   quantity TEXT NULL,
   listing_start_amount TEXT NULL,
   listing_start_date BIGINT NULL,
   buy_in_progress BOOLEAN NOT NULL DEFAULT FALSE,
   has_bid BOOLEAN NOT NULL DEFAULT FALSE,
   held_timestamp BIGINT NULL,
   is_listed BOOLEAN NOT NULL DEFAULT FALSE,
   listing_currency_address TEXT NOT NULL DEFAULT '',
   listing_currency_chain_id TEXT NOT NULL DEFAULT '',
   listing_timestamp BIGINT NULL,
   listing_broker_id TEXT NULL,
   listing_orderhash TEXT NOT NULL DEFAULT '',
   listing_end_amount TEXT NULL,
   listing_end_date BIGINT NULL,
   metadata JSON NULL,
   metadata_ok BOOLEAN NOT NULL DEFAULT FALSE,
   token_id_hex TEXT NOT NULL,
   top_bid_amount TEXT NULL,
   top_bid_broker_id TEXT NULL,
   top_bid_order_hash TEXT NOT NULL DEFAULT '',
   is_burned BOOLEAN NOT NULL DEFAULT FALSE,
   updated_timestamp BIGINT NOT NULL,

   PRIMARY KEY (contract_id, token_id)
);

CREATE TABLE token_events (
  event_id SERIAL PRIMARY KEY,
  order_hash TEXT NOT NULL DEFAULT '',
  contract_id INTEGER NOT NULL,
  token_id TEXT NOT NULL,
  event_type TEXT NOT NULL CHECK (event_type IN ('Listing', 'CollectionOffer', 'Offer', 'Auction', 'Fulfill', 'Cancelled', 'Executed', 'Buy', 'Sell', 'Mint', 'Burn', 'Transfer')),
  timestamp BIGINT NOT NULL,
  token_id_hex TEXT NOT NULL,
  transaction_hash TEXT NULL,
  to_address TEXT NOT NULL, -- NULL if not transfert
  from_address TEXT NOT NULL, -- NULL if new listing
  amount TEXT NOT NULL,
  canceled_reason TEXT,
  FOREIGN KEY (contract_id, token_id) REFERENCES token(contract_id, token_id)
);

CREATE TABLE token_offers (
  offer_id SERIAL PRIMARY KEY,
  contract_id INTEGER NOT NULL,
  token_id TEXT NOT NULL,
  order_hash TEXT NOT NULL DEFAULT '',
  offer_maker TEXT NOT NULL,
  offer_amount TEXT NOT NULL,
  offer_quantity TEXT NOT NULL,
  offer_timestamp BIGINT NOT NULL,
  currency_chain_id TEXT NOT NULL DEFAULT '',
  currency_address TEXT NOT NULL DEFAULT '',
  start_date BIGINT NOT NULL DEFAULT 0,
  end_date BIGINT NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'PLACED',
  FOREIGN KEY (contract_id, token_id) REFERENCES token(contract_id, token_id)
);

CREATE TABLE block (
    block_number SERIAL PRIMARY KEY,
    indexer_identifier TEXT NOT NULL,
    indexer_version TEXT NOT NULL,
    status TEXT NOT NULL,
    timestamp BIGINT NOT NULL
);

CREATE TABLE indexer (
     task_id SERIAL PRIMARY KEY,
     status TEXT NOT NULL,
     last_update BIGINT NOT NULL,
     version TEXT NOT NULL,
     indexation_progress REAL,
     current_block_number BIGINT
);
