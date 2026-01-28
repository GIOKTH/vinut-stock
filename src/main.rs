mod config;
mod db;
mod docs;
mod handlers;
mod middleware;
mod models;
mod routes;
mod security;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use config::Config;
use db::connection_pool;
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let config = Config::init();
    let pool = connection_pool(&config).await;
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Starting server at {}", config.server_address);

    let server_address = config.server_address.clone();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db::AppState {
                db: pool.clone(),
                env: config.clone(),
            }))
            .wrap(Logger::default())
            .wrap(cors)
            .configure(routes::config)
    })
    .bind(server_address)?
    .run()
    .await
}
