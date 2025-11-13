use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub name: String,
    pub avatar_url: String,
    pub created_at: String,
    pub updated_at: String
}

pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub avatar_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}