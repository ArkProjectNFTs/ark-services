-- Disable trigger during modifications
DROP TRIGGER IF EXISTS insert_active_orders ON orders;

-- Update trigger function to include new columns
CREATE OR REPLACE FUNCTION insert_active_orders_function()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO active_orders (
        order_hash, order_type,
        token_address, token_id_hex,
        token_id, start_amount,
        start_amount_eth, end_amount,
        end_date, offerer,
        currency_address, broker_id,
        created_at
    )
    VALUES (
        NEW.order_hash, NEW.order_type,
        NEW.token_address, NEW.token_id_hex,
        NEW.token_id, NEW.start_amount,
        NEW.start_amount_eth, NEW.end_amount,
        NEW.end_date, NEW.offerer,
        NEW.currency_address, NEW.broker_id,
        NEW.created_at
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Reactivate trigger with new function
CREATE TRIGGER insert_active_orders
AFTER INSERT ON orders
FOR EACH ROW
EXECUTE FUNCTION insert_active_orders_function();