# Ark-SQLx

This crate contains drivers written for interacting with relational
databases.

This crates relies on [sqlx crate](https://crates.io/crates/sqlx) to
abstract the interaction with the database. Even if it's not an ORM,
sqlx provides an easy and unified interface to query data from a relational
database.

## Dev setup

Sqlx works with migrations. Those migrations can be found
in the `migrations` folder. Sqlx always run the migration in orders,
and relies on the `DATABASE_URL` that can be found in the `.env` file
if not explicitely given in CLI arguments.

In order to use sqlx from the command line, you can install
[sqlx-cli](https://crates.io/crates/sqlx-cli) crate like so:
```
cargo install sqlx-cli
```

From here, you'll be able to run commands like:
```bash
# Create db and run all pending migrations.
sqlx database setup

# Drop and rebuild a database + run pending migrations.
sqlx database reset

# Run pending migrations only.
sqlx migrate run
```
