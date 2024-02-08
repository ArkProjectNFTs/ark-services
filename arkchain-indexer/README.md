# Arkchain indexer

This indexer aims at indexing all the events from the Orderbook
contract on the Arkchain.

Using Diri under the hood, this crate contains the mainloop and
the application logic to store the Diri indexed data into a postgres database.

# How to run the indexer

To test the indexer locally, you will need several tools installed:
* Katana
* Ark-project crate with Solis
* Docker (to quickly run a database, can be skipped if you already
  have a postgres installed)
* Starkli (to setup the orderbook mocked contract)

1. Run Katana as starknet sequencer:
```
dojoup -v nightly
katana
```

2. Run solis:
```
cd ark-project/

# Addresses does not matter to test
cargo run -p solis -- \
    --messaging crates/solis/messaging.local.json
```

3. Spin up the database and migrations:
```
sudo docker run -d \
    --name arkchain-db \
    -p 5432:5432 \
    -e POSTGRES_PASSWORD=123 \
    postgres
    
cargo install sqlx-cli

cd ark-services/ark-sqlx

sqlx database reset \
    --database-url postgres://postgres:123@localhost:5432/arkchain
```

4. Run some commands to emit mocked events
```
cd ark-project/contracts

# the first time only
make generate_artifacts

make setup_orderbook_events
```

4. Run the indexer:
```
cargo run -p arkchain-indexer
```
