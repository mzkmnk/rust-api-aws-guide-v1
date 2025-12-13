use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::handler::AppState,
    application::services::UserService,
    infrastructure::{config::Config, database::create_pool, repositories::PostgresUserRepository},
};

mod api;
mod application;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Starting server at {}", config.server_addr());

    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connected");

    let user_repo = Arc::new(PostgresUserRepository::new(pool));
    let user_service = Arc::new(UserService::new(user_repo));
    let state = AppState { user_service };

    let app = Router::new()
        .route("/health", get(api::handler::health_check))
        .route(
            "/api/users",
            get(api::handler::list_users).post(api::handler::create_user),
        )
        .route(
            "/api/users/{id}",
            get(api::handler::get_user).delete(api::handler::delete_user),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.server_addr();
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
