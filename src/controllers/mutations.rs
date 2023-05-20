use std::str::FromStr;

use async_graphql::{
    Context, FieldResult, Object, ID
};
use bson::oid::ObjectId;

use crate::models::{Support, UserMessages};

use super::{Storage, StreamChanged, MessageBroker, MutationType};
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

    async fn delete_thread(
        &self,
        ctx: &Context<'_>,
        thread_id: String,
    ) -> FieldResult<Option<Support>> {
        let thread = ctx.data_unchecked::<crate::AppState>().container.support.delete_one(&thread_id).await?;

        Ok(thread)
    }

    async fn send_msg(&self, ctx: &Context<'_>, thread_id: String, username: String, msg: String) -> FieldResult<ID> {
        let mut store = ctx.data_unchecked::<Storage>().lock().and_then(|mutex| Ok(mutex)).unwrap();
		let entry = store.vacant_entry();
		let id: ID = entry.key().into();	
	
		let user = UserMessages {
			id: id.clone().to_string(),
			username: username,
			text: msg,
		};
		entry.insert(user.clone());
        let thread = ctx.data_unchecked::<crate::AppState>().container.support.add_message(&thread_id,user.clone()).await?;
        
        MessageBroker::publish(StreamChanged {
            mutation_type: MutationType::Created,
            id: id.clone(),
        });
        
        Ok(id)
    }
}
