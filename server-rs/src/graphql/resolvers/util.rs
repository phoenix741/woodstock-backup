use async_graphql::ErrorExtensions;

pub fn map_err(e: eyre::Report) -> async_graphql::Error {
    async_graphql::Error::new(e.to_string()).extend_with(|_, extensions| {
        extensions.set("code", "INTERNAL_ERROR");
        extensions.set("message", "An internal error occurred");
    })
}

pub fn find_last_month<T: woodstock::statistics::WithDate>(dates: &[T]) -> Option<&T> {
    let month_ago = chrono::Utc::now() - chrono::Duration::days(30);
    dates
        .iter()
        .find(|d| d.date() >= month_ago)
        .or_else(|| dates.last())
}
