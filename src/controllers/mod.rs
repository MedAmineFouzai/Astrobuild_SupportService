mod mutations;
mod queries;
mod queue;
mod subscription;
use async_graphql::Schema;
pub use mutations::MutationRoot;
pub use queries::QueryRoot;
pub use queue::{Storage, MessageBroker};
pub use subscription::{MutationType, SubscriptionRoot, StreamChanged};
#[derive(Debug)]
pub struct MyToken(pub String);
pub type GraphQlSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
