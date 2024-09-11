-- Create the enumeration types
CREATE TYPE erc_compliance AS ENUM ('OPENZEPPELIN', 'OTHER');
CREATE TYPE erc_action AS ENUM ('MINT', 'BURN', 'OTHER');
CREATE TYPE contract_type AS ENUM ('ERC20', 'ERC721', 'ERC1155', 'ERC1400', 'OTHER');
CREATE TYPE event_type AS ENUM ('Transfer', 'Approval', 'ApprovalForAll', 'TransferSingle', 'TransferBatch', 'URI', 'TransferByPartition', 'ChangedPartition', 'Other');

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS transaction_info (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY, -- UUID primary key
    sequence_id BIGSERIAL,      
    tx_hash VARCHAR(66) NOT NULL,             -- Transaction hash
    event_id VARCHAR(78) NOT NULL,             -- Transaction hash
    from_address VARCHAR(66) NOT NULL,        -- From address
    to_address VARCHAR(66) NOT NULL,          -- Recipient address
    value DECIMAL,                       -- Value as a string (e.g., for large numbers or token amounts)
    timestamp BIGINT NOT NULL,                -- Unix timestamp (in seconds or milliseconds)
    token_id DECIMAL,                    -- Token ID (for ERC721 / ERC1155)
    contract_address VARCHAR(66) NOT NULL,    -- Contract address
    contract_type contract_type NOT NULL,     -- Contract type (e.g., ERC20, ERC721)
    block_hash VARCHAR(66) NOT NULL,          -- Block hash
    event_type event_type NOT NULL,           -- Event Type (e.g., Transfer, Approval ....)
    erc_compliance erc_compliance NOT NULL,   -- ERC Compliance (e.g., OpenZeppelin, other ....)
    erc_action erc_action NOT NULL,           -- ERC Action (e.g., Mint, BURN ....)
    indexed_at TIMESTAMPTZ NOT NULL,          -- Timestamp with timezone for when the entry was indexed
    UNIQUE (tx_hash, event_id) -- Composite primary key to avoid duplicates
);

CREATE TABLE IF NOT EXISTS token_info (
    contract_address VARCHAR(66) PRIMARY KEY,  -- Contract address
    symbol VARCHAR(78) NOT NULL,               -- Token symbol
    decimals SMALLINT NOT NULL,                -- Token decimals
    total_supply DECIMAL,                 -- Total supply as a string (if needed for large values)
    chain_id VARCHAR(66) NOT NULL             -- ID of the blockchain network
);

CREATE TABLE IF NOT EXISTS nft_info (
    contract_address VARCHAR(66) NOT NULL,  -- Contract address is usually a hex string with a length of 255 (0x + 64)
    token_id DECIMAL,                  -- Token ID can vary in length; adjust size as needed
    name VARCHAR(78),
    symbol VARCHAR(78),
    metadata_uri TEXT,
    owner VARCHAR(66) NOT NULL,             -- Address of the owner
    chain_id VARCHAR(66) NOT NULL,         -- ID of the blockchain network
    block_hash VARCHAR(66) NOT NULL,        -- Block hash is a hex string
    indexed_at TIMESTAMPTZ NOT NULL,        -- Timestamp with timezone
    PRIMARY KEY (contract_address, token_id)
);

CREATE TABLE IF NOT EXISTS indexed_blocks (
    block_hash VARCHAR NOT NULL PRIMARY KEY,
    indexed_at TIMESTAMP WITH TIME ZONE NOT NULL
);