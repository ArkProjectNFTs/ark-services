use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use ark_orderbook_api::routes::{default, token};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

/// Initializes the logging, ensuring that the `RUST_LOG` environment
/// variable is always considered first.
fn init_logging() {
    // Initialize the LogTracer to convert `log` records to `tracing` events
    tracing_log::LogTracer::init().expect("Setting log tracer failed.");

    // Create the layers
    let env_filter = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer();

    // Combine layers and set as global default
    let subscriber = Registry::default().with(env_filter).with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let main_span = span!(Level::TRACE, "main");
    let _main_guard = main_span.enter();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_logging();
    let database_url =
        std::env::var("ARKCHAIN_DATABASE_URL").expect("ARKCHAIN_DATABASE_URL must be set");

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
            .configure(default::config)
            .configure(token::config)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
