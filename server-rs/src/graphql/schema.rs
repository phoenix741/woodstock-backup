use async_graphql::http::GraphiQLSource;
use async_graphql::{extensions::ApolloTracing, MergedObject, MergedSubscription, Schema};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{ws::WebSocketUpgrade, Extension, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;

use crate::api::ApiServerState;
use crate::auth::authz::CurrentUser;
use crate::auth::middleware::wait_for_session_revocation;

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
        .route("/graphql/ws", get(graphql_ws_handler))
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

/// Injects the [`CurrentUser`] resolved by `auth::middleware::session_middleware` (which
/// wraps this whole router, see `api::routes::create_router`) into the per-request GraphQL
/// context, so every resolver can enforce host ownership via `ctx.data::<CurrentUser>()`.
async fn graphql_handler(
    Extension(schema): Extension<WoodstockSchema>,
    current_user: CurrentUser,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(current_user))
        .await
        .into()
}

/// Same identity injection as [`graphql_handler`], but for the WebSocket subscription
/// transport: the upgrade request still carries the session cookie (the WS handshake is a
/// plain HTTP request before the protocol switch), so `CurrentUser` extraction works
/// identically here — but since a subscription's `Context` lives for the whole connection,
/// it must be attached once at upgrade time via `GraphQLWebSocket::with_data` rather than
/// per-message, which `GraphQLSubscription::new(schema)` (used before this module existed)
/// does not offer a hook for.
///
/// Unlike REST/HTTP GraphQL requests, an open subscription never goes back through
/// `session_middleware` to notice a logout or an expired session, so it's raced here
/// against [`wait_for_session_revocation`], which closes the connection shortly after the
/// session that authorized it disappears.
async fn graphql_ws_handler(
    State(state): State<ApiServerState>,
    Extension(schema): Extension<WoodstockSchema>,
    current_user: CurrentUser,
    jar: CookieJar,
    protocol: GraphQLProtocol,
    ws: WebSocketUpgrade,
) -> Response {
    let mut data = async_graphql::Data::default();
    data.insert(current_user);

    let sid = state
        .oidc
        .enabled
        .then(|| {
            jar.get(&state.oidc.cookie_name)
                .map(|c| c.value().to_string())
        })
        .flatten();

    // Same protocol list `GraphQLSubscription` (used before this module existed) declared
    // via `ALL_WEBSOCKET_PROTOCOLS` — the frontend's `@apollo/client/link/ws` speaks the
    // legacy `graphql-ws` (subscriptions-transport-ws) subprotocol, which must stay in
    // this list for it to keep negotiating successfully.
    ws.protocols(async_graphql::http::ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |socket| async move {
            let serve = GraphQLWebSocket::new(socket, schema, protocol)
                .with_data(data)
                .serve();
            match sid {
                Some(sid) => {
                    tokio::select! {
                        () = serve => {},
                        () = wait_for_session_revocation(state.redis_client.clone(), sid) => {},
                    }
                }
                None => serve.await,
            }
        })
}
