CREATE TABLE token_balance (
    contract_address TEXT NOT NULL,
    token_id NUMERIC NOT NULL,
    owner_address TEXT NOT NULL,
    balance NUMERIC NOT NULL,
    chain_id TEXT NOT NULL,
    last_updated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (contract_address, token_id, owner_address, chain_id)
);

-- Create an index to improve query performance
CREATE INDEX idx_token_balance_owner ON token_balance(owner_address);
CREATE INDEX idx_token_balance_contract ON token_balance(contract_address); 