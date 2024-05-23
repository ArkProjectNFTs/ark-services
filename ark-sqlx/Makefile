db_url := postgresql://admin:dbpassword@localhost:5433/arkchain

reset_arkchain_db:
	sqlx database reset --database-url ${db_url}
	sqlx migrate run --database-url ${db_url}
