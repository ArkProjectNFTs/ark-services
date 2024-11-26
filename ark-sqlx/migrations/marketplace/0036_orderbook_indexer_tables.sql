-- order route type
CREATE TYPE route_type AS ENUM ('Erc20ToErc721', 'Erc721ToErc20', 'Erc20ToErc1155', 'Erc1155ToErc20');
CREATE TYPE order_type AS ENUM ('Listing', 'Auction', 'Offer', 'CollectionOffer');
CREATE TYPE order_event_type AS ENUM ('Placed', 'Cancelled', 'Fulfilled', 'Executed');
CREATE TYPE order_status AS ENUM ('Open', 'Executed', 'Cancelled');

CREATE TYPE cancelled_reason_type AS ENUM ('CancelledUser', 'CancelledByNewOrder', 'CancelledAssetFault', 'CancelledOwnership', 'Unknown');

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE DOMAIN u256_hex AS VARCHAR(66)
    CHECK (VALUE ~ '^0x[a-fA-F0-9]{64}$');

CREATE TABLE IF NOT EXISTS orders (
    order_hash u256_hex PRIMARY KEY,
    created_at BIGINT NOT NULL,
    route_type route_type NOT NULL,
    order_type order_type NOT NULL,
    currency_address u256_hex NOT NULL,
    currency_chain_id u256_hex NOT NULL,
    offerer u256_hex NOT NULL,
    token_chain_id u256_hex NOT NULL,
    token_address u256_hex NOT NULL,
    token_id u256_hex,
    quantity u256_hex NOT NULL,
    start_amount u256_hex NOT NULL,
    end_amount u256_hex NOT NULL,
    start_date BIGINT NOT NULL,
    end_date BIGINT NOT NULL,
    broker_id u256_hex NOT NULL,
    cancelled_order_hash u256_hex,
    updated_at BIGINT NOT NULL,
    status order_status NOT NULL DEFAULT 'Open'
);

CREATE TABLE IF NOT EXISTS active_orders (
    token_address u256_hex NOT NULL,
    token_id u256_hex,
    order_hash u256_hex NOT NULL,
    order_type order_type NOT NULL,
    start_amount u256_hex NOT NULL,
    end_amount u256_hex NOT NULL,
    end_date BIGINT NOT NULL,
    offerer u256_hex NOT NULL,
    currency_address u256_hex NOT NULL,
    broker_id u256_hex NOT NULL,
    created_at BIGINT NOT NULL,
    PRIMARY KEY (order_hash, token_address)
);

CREATE TABLE IF NOT EXISTS order_transaction_info (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    tx_hash u256_hex NOT NULL,
    event_id BIGINT NOT NULL,
    order_hash u256_hex NOT NULL,
    timestamp BIGINT NOT NULL,
    event_type order_event_type NOT NULL,
    cancelled_reason cancelled_reason_type,
    related_order_hash u256_hex,
    fulfiller u256_hex,
    from_address u256_hex,
    to_address u256_hex,
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

-- Insert new order to active orders
CREATE OR REPLACE FUNCTION insert_active_orders_function()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO active_orders (
        order_hash, order_type,
        token_address, token_id,
        start_amount, end_amount,
        end_date,
        offerer,
        currency_address,
        broker_id,
        created_at
    )
    VALUES (
        NEW.order_hash, NEW.order_type,
        NEW.token_address, NEW.token_id,
        NEW.start_amount, NEW.end_amount,
        NEW.end_date,
        NEW.offerer,
        NEW.currency_address,
        NEW.broker_id,
        NEW.created_at
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER insert_active_orders
AFTER INSERT ON orders
FOR EACH ROW
EXECUTE FUNCTION insert_active_orders_function();

-- Indexes for performance
CREATE INDEX idx_active_orders_token ON active_orders(token_address, token_id);
CREATE INDEX idx_active_orders_end_date ON active_orders(end_date);

CREATE INDEX idx_orders_token ON orders(token_address, token_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_offerer ON orders(offerer);
CREATE INDEX idx_order_events_timestamp ON order_transaction_info(timestamp);
