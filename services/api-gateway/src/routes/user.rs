use std::sync::Arc;
use axum::response::{Response};
use axum::{ Router, extract::Json};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use crate::shared::state::AppState;
use tracing::info;
use crate::models::user::{LoginUser, User};
use crate::response::{ErrorResponse, ResponseResult};

async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginUser>) -> ResponseResult<User> {
    state.req_counter.inc();
    let _timer = state.req_timer.start_timer();
    info!("User login attempt for email: {}", payload.email);
    let user_collection = state.db.collection::<User>("users");
    let user = user_collection.find_one(
        mongodb::bson::doc! { "email": &payload.email }
    ).await.unwrap();
    match user {
        Some(found_user) => {
            if found_user.password == payload.password {
                info!("User {} logged in successfully", found_user.email);
                ResponseResult {
                    success: true,
                    code: StatusCode::OK,
                    error: None,
                    json: Some(Json(found_user)),
                }
            } else {
                info!("User {} provided incorrect password", payload.email);
                ResponseResult {
                    success: false,
                    code: StatusCode::UNAUTHORIZED,
                    error: Some(Json(ErrorResponse { message: "Incorrect password".to_string()})),
                    json: None,
                }
            }
        },
        None => {
            info!("User {} not found", payload.email);
            ResponseResult {
                success: false,
                code: StatusCode::NOT_FOUND,
                error: Some(Json(ErrorResponse { message: "User not found".to_string()})),
                json: None,
            }
        }
    }
}

pub fn register_routes() -> Router<Arc<AppState>> {
    Router::new()
    .route("/login", post(login))
}


