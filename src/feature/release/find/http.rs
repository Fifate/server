use axum::extract::{Path, State};
use axum_extra::extract::Query;
use libfp::BifunctorExt;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ReleaseFilter;
use super::repo::{self, Filter};
use crate::adapter::inbound::rest::api_response::Data;
use crate::adapter::inbound::rest::data;
use crate::adapter::inbound::rest::state::{self, ArcAppState};
use crate::domain::release::Release;
use crate::infra::error::Error;

const TAG: &str = "Release";

data!(
    DataOptionRelease, Option<Release>
    DataVecRelease, Vec<Release>
);

pub fn router() -> OpenApiRouter<ArcAppState> {
    OpenApiRouter::new()
        .routes(routes!(find_release_by_id))
        .routes(routes!(find_release_by_keyword))
        .routes(routes!(find_release_by_filter))
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/release/{id}",
    responses(
        (status = 200, body = DataOptionRelease),
        Error,
    ),
)]
async fn find_release_by_id(
    State(repo): State<state::SeaOrmRepository>,
    Path(id): Path<i32>,
) -> Result<Data<Option<Release>>, Error> {
    repo::find_one(&repo, Filter::Id(id)).await.bimap_into()
}

#[derive(IntoParams, Deserialize)]
struct KwQuery {
    keyword: String,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/release",
    params(KwQuery),
    responses(
        (status = 200, body = DataVecRelease),
        Error,
    ),
)]
async fn find_release_by_keyword(
    State(repo): State<state::SeaOrmRepository>,
    Query(query): Query<KwQuery>,
) -> Result<Data<Vec<Release>>, Error> {
    repo::find_many(&repo, Filter::Keyword(query.keyword))
        .await
        .bimap_into()
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/release/filter",
    params(ReleaseFilter),
    responses(
        (status = 200, body = DataVecRelease),
        Error,
    ),
)]
async fn find_release_by_filter(
    State(repo): State<state::SeaOrmRepository>,
    Query(filter): Query<ReleaseFilter>,
) -> Result<Data<Vec<Release>>, Error> {
    let release_types = filter.release_types.unwrap_or_default();
    repo::find_many(&repo, Filter::ReleaseTypes(release_types))
        .await
        .bimap_into()
}
