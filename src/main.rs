#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;

use crate::slack::handler::handler;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use dotenv::dotenv;
use env_logger::Env;
use r2d2::Pool;

mod schema;
mod slack;
mod sotw_db;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Create a DB connection pool using simple defaults
pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    let connection = std::env::var("DATABASE_URL").expect("Database connection string missing!");
    let manager = ConnectionManager::<PgConnection>::new(connection);

    r2d2::Pool::builder()
        .max_size(2)
        .build(manager)
        .expect("Failed creating connection pool!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let log_level = std::env::var("LOG_LEVEL").unwrap_or("info".to_string());
    env_logger::from_env(Env::default().default_filter_or(log_level)).init();

    let db_pool = create_db_pool();

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .wrap(Logger::default())
            .route("/", web::post().to(handler))
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}
