use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

use crate::app::{
    AppState,
    api::{openapi, router},
};

const ADMIN_TOKEN: &str = "flare_test_admin";
const ID: &str = "00000000-0000-0000-0000-000000000000";

// the lazy pool never connects, so these tests only cover paths that stop before a query
fn app() -> Router {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused")
        .unwrap();
    router(AppState {
        pool,
        admin_token: ADMIN_TOKEN.to_string(),
    })
}

async fn status(method: &str, path: &str, token: Option<&str>) -> StatusCode {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let request = request.body(Body::empty()).unwrap();
    app().oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn admin_routes_reject_bad_tokens() {
    let routes = [
        ("POST", "/v1/jobs".to_string()),
        ("GET", "/v1/jobs".to_string()),
        ("GET", format!("/v1/jobs/{ID}")),
        ("PATCH", format!("/v1/jobs/{ID}")),
        ("DELETE", format!("/v1/jobs/{ID}")),
        ("GET", format!("/v1/jobs/{ID}/token")),
        ("GET", "/v1/workers".to_string()),
        ("GET", format!("/v1/workers/{ID}")),
        ("PUT", format!("/v1/workers/{ID}")),
        ("POST", format!("/v1/workers/{ID}/heartbeat")),
        ("GET", "/v1/runs".to_string()),
        ("GET", format!("/v1/runs/{ID}")),
        ("GET", format!("/v1/runs/{ID}/events")),
    ];
    // same length as the real token, then a prefix of it
    let tokens = [None, Some("flare_test_wrong"), Some(&ADMIN_TOKEN[..5])];

    for (method, path) in &routes {
        for token in tokens {
            let got = status(method, path, token).await;
            assert_eq!(got, StatusCode::UNAUTHORIZED, "{method} {path} {token:?}");
        }
    }
}

// 415 comes from the Json extractor in the handler, so auth let the request through
#[tokio::test]
async fn admin_token_reaches_handlers() {
    for (method, path) in [
        ("POST", "/v1/jobs".to_string()),
        ("PUT", format!("/v1/workers/{ID}")),
    ] {
        let got = status(method, &path, Some(ADMIN_TOKEN)).await;
        assert_eq!(got, StatusCode::UNSUPPORTED_MEDIA_TYPE, "{method} {path}");
    }
}

#[tokio::test]
async fn job_routes_require_a_token() {
    for (method, path) in [
        ("PUT", format!("/v1/runs/{ID}")),
        ("POST", format!("/v1/runs/{ID}/heartbeat")),
        ("POST", format!("/v1/runs/{ID}/finish")),
        ("PUT", format!("/v1/runs/{ID}/steps/{ID}")),
        ("POST", format!("/v1/runs/{ID}/steps/{ID}/finish")),
    ] {
        let got = status(method, &path, None).await;
        assert_eq!(got, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
}

#[tokio::test]
async fn health_is_public() {
    assert_eq!(status("GET", "/v1/health", None).await, StatusCode::OK);
}

#[test]
fn openapi_spec_is_up_to_date() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../api/openapi.json");
    let spec = openapi().to_pretty_json().unwrap() + "\n";

    if std::env::var_os("UPDATE_OPENAPI").is_some() {
        std::fs::write(&path, &spec).unwrap();
        return;
    }

    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == spec,
        "api/openapi.json is out of date, run `make openapi`"
    );
}
