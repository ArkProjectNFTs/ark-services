## DynamoDB Table Schema

### Token Entity
- **PK**: `TOKEN#<contract_address>#<token_id>`
- **SK**: `TOKEN`
- **Type**: `Token`
- **GSI1PK**: `COLLECTION#<contract_address>`
- **GSI1SK**: `TOKEN#<token_id>`
- **GSI2PK**: `OWNER#<owner_address>`
- **GSI2SK**: `TOKEN#<contract_address>#<token_id>`
- **GSI3PK**: `LISTED#<true/false>`
- **GSI3SK**: `TOKEN#<contract_address>#<token_id>`
- **GSI4PK**: `BLOCK#<block_number>`
- **GSI4SK**: `TOKEN#<contract_address>#<token_id>`
- **Data**: 
  - `owner`: `<owner_address>`
  - `mint_address`: `<mint_address>`
  - `is_listed`: `<true/false>`
  - `...`: `...`

### Collection Entity
- **PK**: `COLLECTION#<contract_address>`
- **SK**: `COLLECTION`
- **Type**: `Collection`
- **GSI4PK**: `BLOCK#<block_number>`
- **GSI4SK**: `COLLECTION#<contract_address>`
- **Data**: 
  - `contract_type`: `<ERC721/ERC1155/...>`
  - `...`: `...`

### Event Entity
- **PK**: `EVENT#<contract_address>#<event_id>`
- **SK**: `EVENT`
- **Type**: `Event`
- **GSI2PK**: `TOKEN#<token_id>`
- **GSI2SK**: `EVENT#<event_id>`
- **GSI4PK**: `BLOCK#<block_number>`
- **GSI4SK**: `EVENT#<contract_address>#<event_id>`
- **Data**: 
  - `from_address`: `<from_address>`
  - `to_address`: `<to_address>`
  - `event_type`: `<Mint/Burn/Transfer/...>`
  - `...`: `...`

### Block Entity
- **PK**: `BLOCK#<block_number>`
- **SK**: `BLOCK`
- **Type**: `Block`
- **GSI4PK**: `BLOCK#<block_number>`
- **GSI4SK**: `BLOCK`
- **Data**: 
  - `indexer_version`: `<version_number>`
  - `status`: `<status>`
  - `...`: `...`
