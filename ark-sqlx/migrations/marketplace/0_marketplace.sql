CREATE TABLE broker (
    id TEXT PRIMARY KEY,
    contract_address VARCHAR(66) NOT NULL,
    chain_id TEXT NOT NULL,
    name TEXT NOT NULL,
    UNIQUE (contract_address, chain_id)
);

CREATE TABLE contract (
  contract_address VARCHAR(66) NOT NULL,
  chain_id TEXT NOT NULL,
  updated_timestamp BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
  floor_price TEXT,
  top_bid TEXT,
  contract_type TEXT NOT NULL CHECK (contract_type IN ('ERC721', 'ERC1155', 'OTHER')),
  contract_name TEXT,
  contract_symbol TEXT,
  contract_image TEXT,
  metadata_ok BOOLEAN NOT NULL DEFAULT FALSE,
  is_spam BOOLEAN NOT NULL DEFAULT FALSE,
  is_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
  
  PRIMARY KEY (contract_address, chain_id)
);

CREATE TABLE token (
  contract_address VARCHAR(66) NOT NULL,
  chain_id TEXT NOT NULL,
  token_id TEXT NOT NULL,
  token_id_hex TEXT NOT NULL,
  current_owner TEXT,
  last_price TEXT NULL,
  currency_address TEXT,
  currency_chain_id TEXT,
  status TEXT,
  quantity TEXT NULL,
  listing_start_amount TEXT NULL,
  listing_start_date BIGINT NULL,
  buy_in_progress BOOLEAN NOT NULL DEFAULT FALSE,
  has_bid BOOLEAN NOT NULL DEFAULT FALSE,
  held_timestamp BIGINT NULL,
  is_listed BOOLEAN NOT NULL DEFAULT FALSE,
  listing_currency_address TEXT,
  listing_currency_chain_id TEXT,
  listing_timestamp BIGINT NULL,
  listing_broker_id TEXT NULL,
  listing_orderhash TEXT,
  listing_end_amount TEXT NULL,
  listing_end_date BIGINT NULL,
  metadata JSON NULL,
  metadata_ok BOOLEAN NOT NULL DEFAULT FALSE,
  top_bid_amount TEXT NULL,
  top_bid_broker_id TEXT NULL,
  top_bid_order_hash TEXT,
  is_burned BOOLEAN NOT NULL DEFAULT FALSE,
  block_timestamp BIGINT NOT NULL,
  updated_timestamp BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),

  PRIMARY KEY (contract_address, chain_id, token_id),
  FOREIGN KEY (contract_address, chain_id) REFERENCES contract(contract_address, chain_id) ON DELETE RESTRICT ON UPDATE CASCADE,
  FOREIGN KEY (listing_broker_id) REFERENCES broker(id) ON DELETE RESTRICT ON UPDATE CASCADE,
  FOREIGN KEY (top_bid_broker_id) REFERENCES broker(id) ON DELETE RESTRICT ON UPDATE CASCADE
);

CREATE TABLE token_event (
  token_event_id TEXT PRIMARY KEY,
  contract_address VARCHAR(66) NOT NULL,
  chain_id TEXT NOT NULL,
  broker_id TEXT,
  order_hash TEXT,
  token_id TEXT NOT NULL,
  token_id_hex TEXT NOT NULL,
  event_type TEXT CHECK (event_type IN 
    ('Listing', 'CollectionOffer', 'Offer', 'Auction', 
      'Fulfill', 'Cancelled', 'Executed', 'Sale', 'Mint', 
      'Burn', 'Transfer', 'ListingCancelled', 'AuctionCancelled', 'OfferCancelled')),
  block_timestamp BIGINT NOT NULL,
  transaction_hash TEXT NULL,
  to_address TEXT, -- NULL if not transfer
  from_address TEXT, -- NULL if new listing
  amount TEXT,
  canceled_reason TEXT,
  FOREIGN KEY (contract_address, chain_id, token_id) REFERENCES token(contract_address, chain_id, token_id),
  FOREIGN KEY (broker_id) REFERENCES broker(id)
);

CREATE TABLE token_offer (
  token_offer_id SERIAL PRIMARY KEY,
  contract_address VARCHAR(66) NOT NULL,
  chain_id TEXT NOT NULL,
  broker_id TEXT,
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
  FOREIGN KEY (contract_address, chain_id, token_id) REFERENCES token(contract_address, chain_id, token_id),
  FOREIGN KEY (broker_id) REFERENCES broker(id)
);

CREATE TABLE indexer (
  indexer_identifier TEXT PRIMARY KEY,
  indexer_status TEXT,
  last_updated_timestamp BIGINT,
  created_timestamp BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
  indexer_version TEXT, 
  indexation_progress_percentage NUMERIC(5, 2) DEFAULT 0.00,
  current_block_number BIGINT,
  is_force_mode_enabled BOOLEAN NOT NULL DEFAULT FALSE, 
  start_block_number BIGINT, 
  end_block_number BIGINT
);

CREATE TABLE block (
  block_number BIGINT PRIMARY KEY,
  block_status TEXT,
  block_timestamp BIGINT NOT NULL, 
  indexer_identifier TEXT,
  FOREIGN KEY (indexer_identifier) REFERENCES indexer(indexer_identifier)
);

CREATE INDEX idx_token_current_owner ON token(current_owner);
CREATE INDEX idx_token_status ON token(status);
CREATE INDEX idx_token_event_block_timestamp ON token_event(block_timestamp);
CREATE INDEX idx_token_offer_offer_timestamp ON token_offer(offer_timestamp);
