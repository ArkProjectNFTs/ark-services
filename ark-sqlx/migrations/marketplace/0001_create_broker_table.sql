CREATE TABLE broker (
    contract_address TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (contract_address)
);

ALTER TABLE token
ADD CONSTRAINT fk_top_bid_broker_id
FOREIGN KEY (top_bid_broker_id)
REFERENCES broker (contract_address)
ON DELETE SET NULL;  

ALTER TABLE token
ADD CONSTRAINT fk_listing_broker_id
FOREIGN KEY (listing_broker_id)
REFERENCES broker (contract_address)
ON DELETE SET NULL; 
