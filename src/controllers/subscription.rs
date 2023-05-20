use crate::models::UserMessages;
use async_graphql::{Context, Enum, Object, Result, Subscription, ID};
use futures::{Stream, StreamExt};

use super::{queue::MessageBroker, Storage};

pub struct SubscriptionRoot;

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
enum MutationType {
    Created,
}

#[derive(Clone)]
struct StreamChanged {
    mutation_type: MutationType,
    id: ID,
}

#[Object]
impl StreamChanged {
    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn id(&self) -> &ID {
        &self.id
    }

    async fn user_message(&self, ctx: &Context<'_>) -> Result<Option<UserMessages>> {
        let id = self.id.parse::<usize>()?;
        let message = ctx
            .data_unchecked::<Storage>()
            .lock()
            .and_then(|mutex| Ok(mutex.get(id).cloned()));
        Ok(message.unwrap())
    }
}

#[Subscription]
impl SubscriptionRoot {
    async fn connect_stream(
        &self,
        mutation_type: Option<MutationType>,
    ) -> impl Stream<Item = StreamChanged> {
        MessageBroker::<StreamChanged>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }
}
