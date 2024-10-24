-- order route type
CREATE TYPE route_type AS ENUM ('Erc20ToErc721', 'Erc721ToErc20', 'Erc20ToErc1155', 'Erc1155ToErc20');
CREATE TYPE order_type AS ENUM ('Listing', 'Auction', 'Offer', 'CollectionOffer');
CREATE TYPE order_event_type AS ENUM ('Placed', 'Cancelled', 'Fulfilled', 'Executed');
CREATE TYPE cancelled_reason_type AS ENUM ('CancelledUser', 'CancelledByNewOrder', 'CancelledAssetFault', 'CancelledOwnership', 'Unknown');

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS orders (
    hash VARCHAR(66) NOT NULL,
    route_type route_type NOT NULL,
    order_type order_type NOT NULL,
    currency_address VARCHAR(66) NOT NULL,
    currency_chain_id VARCHAR(66) NOT NULL,
    offerer VARCHAR(66) NOT NULL,
    token_chain_id VARCHAR(66) NOT NULL,
    token_address VARCHAR(66) NOT NULL,
    token_id VARCHAR(66),
    quantity VARCHAR(66) NOT NULL,
    start_amount VARCHAR(66) NOT NULL,
    end_amount VARCHAR(66) NOT NULL,
    start_date BIGINT NOT NULL,
    end_date BIGINT NOT NULL,
    broker_id VARCHAR(66) NOT NULL,
    cancelled_order_hash VARCHAR(66)
);

CREATE TABLE IF NOT EXISTS order_transaction_info (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    event_id BIGINT NOT NULL,
    order_hash VARCHAR(66) NOT NULL,
    timestamp BIGINT NOT NULL,
    event_type order_event_type NOT NULL,
    cancelled_reason cancelled_reason_type,
    related_order_hash VARCHAR(66),
    fulfiller VARCHAR(66),
    from_address VARCHAR(66),
    to_address VARCHAR(66),
    CONSTRAINT check_fulfilled_has_fulfiller
        CHECK (CASE 
            WHEN event_type = 'Fulfilled' THEN fulfiller IS NOT NULL
            ELSE TRUE
        END),
    CONSTRAINT check_cancelled_has_reason
        CHECK (CASE 
            WHEN event_type = 'Cancelled' THEN cancelled_reason IS NOT NULL
            ELSE TRUE
        END),
    CONSTRAINT check_executed_has_from_address
        CHECK (CASE 
            WHEN event_type = 'Executed' THEN from_address IS NOT NULL
            ELSE TRUE
        END),
    CONSTRAINT check_executed_has_to_address
        CHECK (CASE 
            WHEN event_type = 'Executed' THEN to_address IS NOT NULL
            ELSE TRUE
        END),

    UNIQUE (tx_hash, event_id)
);


