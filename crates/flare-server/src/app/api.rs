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
fn require(spec: &mut utoipa::openapi::OpenApi, scheme: &str) {
    for item in spec.paths.paths.values_mut() {
        for op in operations_mut(item) {
            op.security = Some(vec![utoipa::openapi::SecurityRequirement::new(
                scheme,
                Vec::<String>::new(),
            )]);
        }
    }
}

#[cfg(test)]
fn operations_mut(
    item: &mut utoipa::openapi::PathItem,
) -> impl Iterator<Item = &mut utoipa::openapi::path::Operation> {
    [
        &mut item.get,
        &mut item.put,
        &mut item.post,
        &mut item.delete,
        &mut item.patch,
        &mut item.head,
        &mut item.options,
        &mut item.trace,
        &mut item.query,
    ]
    .into_iter()
    .flatten()
}

#[cfg(test)]
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::openapi::OpenApiBuilder;
    use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

    let admin_schema = SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .description(Some("Admin token for admin panel"))
            .build(),
    );

    let job_schema = SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .description(Some("Job token for running specific job"))
            .build(),
    );

    let mut admin = admin_api().into_openapi();
    require(&mut admin, "admin_token");

    let mut job = job_api().into_openapi();
    require(&mut job, "job_token");

    admin.merge(job);

    let mut spec = OpenApiBuilder::new().build().nest(PREFIX, admin);
    spec.info = utoipa::openapi::Info::new("Flare", env!("CARGO_PKG_VERSION"));

    let components = spec.components.get_or_insert_with(Default::default);
    components.add_security_scheme("admin_token", admin_schema);
    components.add_security_scheme("job_token", job_schema);

    spec
}
