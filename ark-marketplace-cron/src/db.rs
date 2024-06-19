use sqlx::{PgPool, postgres::PgPoolOptions};
use serde::Deserialize;
use aws_sdk_secretsmanager::Client;
use std::env;

#[derive(Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
    host: String,
    port: u16,
    dbname: String,
}

async fn get_database_url() -> Result<String, Box<dyn std::error::Error>> {
    match env::var("DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_name = env::var("AWS_SECRET_NAME").expect("AWS_SECRET_NAME not set");
            let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            let client = Client::new(&config);
            let secret_value = client
                .get_secret_value()
                .secret_id(secret_name)
                .send()
                .await?;
            let result = secret_value.secret_string().unwrap();

            let creds: DatabaseCredentials = serde_json::from_str(&result)?;
            let database_url = format!(
                "postgres://{}:{}@{}:{}/{}",
                creds.username, creds.password, creds.host, creds.port, creds.dbname
            );

            Ok(database_url)
        }
    }
}

pub async fn get_db_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = get_database_url().await?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    Ok(pool)
}
