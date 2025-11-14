use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

/// Swagger UI HTML page
const SWAGGER_UI_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Documentation - User Authentication Service</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.10.0/swagger-ui.css">
    <style>
        body {
            margin: 0;
            padding: 0;
        }
        .topbar {
            display: none;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.0/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.0/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            window.ui = SwaggerUIBundle({
                url: "/docs/openapi.yaml",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                defaultModelsExpandDepth: 1,
                defaultModelExpandDepth: 1,
                docExpansion: "list",
                filter: true,
                showRequestHeaders: true,
                tryItOutEnabled: true
            });
        };
    </script>
</body>
</html>
"#;

/// Handler for Swagger UI page
async fn swagger_ui() -> impl IntoResponse {
    Html(SWAGGER_UI_HTML)
}

/// Handler for OpenAPI specification
async fn openapi_spec() -> Response {
    // Read the OpenAPI spec file
    match std::fs::read_to_string("services/api-gateway/openapi/openapi.yaml") {
        Ok(content) => (
            StatusCode::OK,
            [("content-type", "application/x-yaml")],
            content,
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to read OpenAPI spec: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load OpenAPI specification",
            )
                .into_response()
        }
    }
}

/// Handler for API documentation redirect
async fn docs_redirect() -> impl IntoResponse {
    axum::response::Redirect::permanent("/docs/ui")
}

/// Create documentation router
pub fn create_docs_router() -> Router {
    Router::new()
        .route("/docs", get(docs_redirect))
        .route("/docs/", get(docs_redirect))
        .route("/docs/ui", get(swagger_ui))
        .route("/docs/openapi.yaml", get(openapi_spec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_swagger_ui_endpoint() {
        let app = create_docs_router();

        let response = app
            .oneshot(Request::builder().uri("/docs/ui").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_docs_redirect() {
        let app = create_docs_router();

        let response = app
            .oneshot(Request::builder().uri("/docs").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    }
}
