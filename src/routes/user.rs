use std::sync::Arc;
use axum::response::IntoResponse;
use axum::{ Router, extract::Json};
use axum::extract::State;
use axum::routing::post;
use crate::shared::state::AppState;
use tracing::info;
use crate::models::user::{LoginUser, User};

async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginUser>) -> impl IntoResponse {
    state.req_counter.inc();
    let _timer = state.req_timer.start_timer();
    info!("User login attempt for email: {}", payload.email);
    let user_collection = state.db.collection::<User>("users");
    let user = user_collection.find_one(
        mongodb::bson::doc! { "email": &payload.email }
    ).await;
    match user {
        Ok(Some(found_user)) => {
            if found_user.password == payload.password {
                info!("User {} logged in successfully", payload.email);
                (axum::http::StatusCode::OK, "Login successful").into_response()
            } else {
                info!("User {} provided incorrect password", payload.email);
                (axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
            }
        },
        Ok(None) => {
            info!("User {} not found", payload.email);
            (axum::http::StatusCode::NOT_FOUND, "User not found").into_response()
        },
        Err(e) => {
            info!("Database error during login for user {}: {}", payload.email, e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

pub fn register_routes() -> Router<Arc<AppState>> {
    Router::new()
    .route("/login", post(login))
}


