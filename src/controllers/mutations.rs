use std::str::FromStr;

use async_graphql::{
    Context, Enum, Error as graphql_error, FieldResult, Object, Result, Schema, Subscription, ID,
};
use bson::oid::ObjectId;

use crate::models::Support;
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_thread(
        &self,
        ctx: &Context<'_>,
        project_id: String,
        title: String,
        thread_description: String,
    ) -> FieldResult<Support> {
        let thread = Support {
            _id: Some(ObjectId::new()),
            project_id: Some(ObjectId::from_str(&project_id)?),
            title: title,
            thread_description: thread_description,
            user_messages: vec![],
        };
        ctx.data_unchecked::<crate::AppState>()
            .container
            .support
            .insert_one(&thread)
            .await?;
        Ok(thread)
    }
}
