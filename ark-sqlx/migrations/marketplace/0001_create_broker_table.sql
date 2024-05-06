CREATE TABLE broker (
    contract_address CHAR(64) NOT NULL,
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

ALTER TABLE token_event
ADD COLUMN broker_address CHAR(64);

ALTER TABLE token_event
ADD CONSTRAINT fk_broker
FOREIGN KEY (broker_address)
REFERENCES broker (contract_address)
ON DELETE SET NULL;