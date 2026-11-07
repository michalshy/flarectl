use axum::Router;
use axum::middleware;
use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::app::AppState;
use crate::app::auth::AuthAdmin;
use crate::app::routes;

const PREFIX: &str = "/v1";

fn admin_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().merge(routes::runs::admin_api())
}

fn job_api() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().merge(routes::runs::job_api())
}

pub fn router(state: AppState) -> Router {
    let (admin_documented, _) = admin_api().split_for_parts();
    let (job_documented, _) = job_api().split_for_parts();

    let admin = Router::new()
        .merge(admin_documented)
        .merge(routes::jobs::router())
        .merge(routes::workers::router())
        .route_layer(middleware::from_extractor_with_state::<AuthAdmin, _>(
            state.clone(),
        ));

    let v1 = Router::new()
        .merge(admin)
        .merge(job_documented)
        .merge(routes::steps::router())
        .route("/health", get(routes::health));

    Router::new().nest(PREFIX, v1).with_state(state)
}

#[cfg(test)]
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut spec = OpenApiRouter::<AppState>::new()
        .nest(PREFIX, admin_api().merge(job_api()))
        .into_openapi();
    spec.info = utoipa::openapi::Info::new("Flare", env!("CARGO_PKG_VERSION"));
    spec
}
