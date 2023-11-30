db_url := postgres://postgres:123@localhost:5432/arkchain

reset_arkchain_db:
	sqlx database reset --database-url ${db_url}
	sqlx migrate run --database-url ${db_url}
