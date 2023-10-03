## Database Schema

### 1. Token Entity

**Primary Key:**

- **PK:** `TOKEN#<contract_address>#<token_id>`
- **SK:** `TOKEN`

**Attributes:**

- **Type:** "Token"
- **Owner:** `<owner_address>`
- **MintAddress:** (if available) `<mint_address>`
- **MintTimestamp:** (if available) `<mint_timestamp>`

**GSI1 (for collection queries):**

- **GSI1PK:** `COLLECTION#<contract_address>`
- **GSI1SK:** `TOKEN#<token_id>`

**GSI2 (for owner queries):**

- **GSI2PK:** `OWNER#<owner_address>`
- **GSI2SK:** `TOKEN#<contract_address>#<token_id>`

---

### 2. Collection Entity

**Primary Key:**

- **PK:** `COLLECTION#<contract_address>`
- **SK:** `COLLECTION`

**Attributes:**

- **Type:** "Collection"
- **ContractType:** (e.g., "ERC721")

**GSI1 (for owner queries):**

- **GSI1PK:** `OWNER#<owner_address>`
- **GSI1SK:** `COLLECTION#<contract_address>`

---

### 3. Event Entity

**Primary Key:**

- **PK:** `EVENT#<contract_address>#<event_id>`
- **SK:** `EVENT`

**Attributes:**

- **Type:** "Event"
- **EventType:** (e.g., "Transfer", "Mint", etc.)
- **FromAddress:** `<from_address>`
- **ToAddress:** `<to_address>`
- **Timestamp:** `<timestamp>`
- **TransactionHash:** `<transaction_hash>`
- **TokenId:** `<token_id>`

**GSI1 (for collection queries):**

- **GSI1PK:** `COLLECTION#<contract_address>`
- **GSI1SK:** `EVENT#<event_id>`

**GSI2 (for owner queries):**

- **GSI2PK:** `OWNER#<from_address>` or `OWNER#<to_address>`
- **GSI2SK:** `EVENT#<contract_address>#<event_id>`
