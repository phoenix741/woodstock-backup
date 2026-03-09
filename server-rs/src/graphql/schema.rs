use async_graphql::http::GraphiQLSource;
use async_graphql::{extensions::ApolloTracing, MergedObject, MergedSubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::api::ApiServerState;

use super::progress::subscriptions::ProgressSubscription;
use super::resolvers::{MutationRoot, QueryRoot};

#[derive(MergedObject, Default)]
pub struct QueryMerged(QueryRoot);

#[derive(MergedSubscription, Default)]
pub struct SubscriptionMerged(ProgressSubscription);

pub type WoodstockSchema = Schema<QueryMerged, MutationRoot, SubscriptionMerged>;

pub fn build_schema(state: ApiServerState) -> WoodstockSchema {
    Schema::build(
        QueryMerged::default(),
        MutationRoot::default(),
        SubscriptionMerged::default(),
    )
    .extension(ApolloTracing)
    .data(state)
    .finish()
}

pub fn graphql_router(schema: WoodstockSchema) -> Router<ApiServerState> {
    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route_service("/graphql/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql/ws")
            .finish(),
    )
}

async fn graphql_handler(
    Extension(schema): Extension<WoodstockSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
