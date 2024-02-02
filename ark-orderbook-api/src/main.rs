use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use routes::token;
use sqlx::postgres::PgPoolOptions;

mod db;
mod handlers;
mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
