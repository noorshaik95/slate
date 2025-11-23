//! Response type definitions.
//!
//! These types are part of the public API for responses.

#![allow(dead_code)]

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}
pub struct ResponseResult<T> {
    pub code: StatusCode,
    pub success: bool,
    pub error: Option<Json<ErrorResponse>>,
    pub json: Option<Json<T>>,
}
impl<T> IntoResponse for ResponseResult<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self.error {
            Some(err_msg) => (self.code, err_msg).into_response(),
            None => match self.json {
                None => self.code.into_response(),
                Some(json_body) => (self.code, json_body.into_response()).into_response(),
            },
        }
    }
}
