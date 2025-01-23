ALTER TABLE orders
ADD COLUMN start_amount_eth NUMERIC;

ALTER TABLE active_orders 
ADD COLUMN start_amount_eth NUMERIC;
