use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use routes::{default, token};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;
mod db;
mod handlers;
mod models;
mod routes;
mod utils;


/// Initializes the logging, ensuring that the `RUST_LOG` environment
/// variable is always considered first.
fn init_logging() {
    const DEFAULT_LOG_FILTER: &str = "trace";

    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .or(EnvFilter::try_new(DEFAULT_LOG_FILTER))
                    .expect("Invalid RUST_LOG filters"),
            )
            .finish(),
    )
        .expect("Failed to set the global tracing subscriber");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_logging();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    trace!("Connecting to : {:?}", database_url);

    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Could not connect to the database");

    HttpServer::new(move || {
        let cors = Cors::default()
            // Maybe we need to add some origin for security reason.
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db_pool.clone()))
            .configure(token::config)
            .configure(default::config)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
