use juniper::{EmptyMutation, EmptySubscription, FieldResult, IntoFieldError, RootNode};

use super::model;
use crate::state;

#[derive(Debug, Clone)]
pub struct Context {
    pub state: state::State,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::graphql_object(
    Context = Context
)]
impl Query {
    /// Return a list of all features
    async fn status(&self, context: &Context) -> FieldResult<model::BragiInfoResponseBody> {
        let bragi_url = format!(
            "http://{}:{}",
            context.state.settings.bragi.host, context.state.settings.bragi.port
        );
        model::status(&bragi_url)
            .await
            .map_err(IntoFieldError::into_field_error)
            .into()
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}
