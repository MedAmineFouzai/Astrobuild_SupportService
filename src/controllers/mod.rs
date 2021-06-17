mod simple_broker;
use bson::oid::ObjectId;
use simple_broker::SimpleBroker;
mod schema;

use async_graphql::{Context,Error as graphql_error, Enum, FieldResult, ID, Object, Result, Schema, Subscription};
use futures::lock::Mutex;
use futures::{Stream, StreamExt};
use slab::Slab;
use std::{sync::Arc};
use crate::{AppState, controllers::schema::{
	UserMessageObject,FileOutput,FileInput,ThreadObject,Thread,ThreadSerializeObject,ThreadDeserializeObject}};



#[derive(Debug)]
pub struct MyToken(pub String);


pub type MessageSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub type Storage = Arc<Mutex<Slab<UserMessageObject>>>;

pub struct QueryRoot;



#[Object]
impl QueryRoot {

	async fn messages(&self, ctx: &Context<'_>,thread_id:String) -> FieldResult<Vec<UserMessageObject>> {

		match ctx.data_unchecked::<crate::AppState>().container.support.find_one_by_id(&thread_id).await
        .and_then(|document| {
            Ok(match document {
                Some(doc) => doc,
                None => bson::Document::new(),
            })
        }) {
        Ok(document) => match document {
            document => match bson::from_document::<ThreadDeserializeObject>(document) {
                Ok(thread) => {
				let thread=ThreadSerializeObject::build_from(thread);
				Ok(thread.user_messages.unwrap())
                }
                Err(_bson_de_error) => Err(graphql_error::new("No Threads Found")),
            },
        },
        Err(_mongodb_error) => Err(graphql_error::new("Server Error")),
    }
		
		// let messages = ctx.data_unchecked::<Storage>().lock().await;
		// Ok(messages.iter().map(|(_, msg)| msg).cloned().collect())

	}

	async fn get_project_threads(&self, ctx: &Context<'_>,project_id:String) -> FieldResult<Vec<ThreadObject>> {

		match ctx.data_unchecked::<crate::AppState>().container.support.find_all_threads_for_project(&project_id).await{
    		Ok(cursor) => {

				let threads: Vec<ThreadSerializeObject> = cursor
                .map(|doc| {
                    let thread =
                        bson::from_document::<ThreadDeserializeObject>(match doc {
                            Ok(thread) => match thread {
                                thread => thread,
                            },
                            Err(_mongodb_error) => bson::Document::new(),
                        })
                        .ok();
						ThreadSerializeObject::build_from(thread.unwrap())
                })
                .collect::<Vec<ThreadSerializeObject>>()
                .await;
				Ok(threads.into_iter().map(|thread|{
					ThreadObject{
        				id: thread.id,
       					title: thread.title,
        				thread_description: thread.thread_description,
        				user_messages: thread.user_messages,
					}
				}).collect::<Vec<ThreadObject>>())
				
			}
    		Err(mongodb_error) => {
				Err(graphql_error::new("No Threads Found"))
			}

		}

	}

	async fn get_Thread_by_id(&self, ctx: &Context<'_>,thread_id:String) -> FieldResult<ThreadObject> {

		match ctx.data_unchecked::<crate::AppState>().container.support.find_thread_by_id(&thread_id).await{
    		Ok(cursor) => {

				let threads: Vec<ThreadSerializeObject> = cursor
                .map(|doc| {
                    let thread =
                        bson::from_document::<ThreadDeserializeObject>(match doc {
                            Ok(thread) => match thread {
                                thread => thread,
                            },
                            Err(_mongodb_error) => bson::Document::new(),
                        })
                        .ok();
						ThreadSerializeObject::build_from(thread.unwrap())
                })
                .collect::<Vec<ThreadSerializeObject>>()
                .await;
				let thread=threads.into_iter().map(|thread|{
					ThreadObject{
        				id: thread.id,
       					title: thread.title,
        				thread_description: thread.thread_description,
        				user_messages: thread.user_messages,
					}
				}).collect::<Vec<ThreadObject>>().last().unwrap().clone();
				Ok(thread)
				
			}
    		Err(mongodb_error) => {
				Err(graphql_error::new("No Threads Found"))
			}

		}

	}




}

pub struct MutationRoot;

#[Object]
impl MutationRoot {

	async fn create_thread(&self, ctx: &Context<'_>,project_id:String,title:String,thread_description:String)->FieldResult<ID>{

		match ctx.data_unchecked::<crate::AppState>().container.support.insert_one(Thread{
			project_id:ObjectId::with_string(&project_id).unwrap(),
			title:title,
			thread_description:thread_description,
			user_messages:Some(vec![])
		}).await
        .and_then(|insret_one_result| {
			Ok(insret_one_result.inserted_id.as_object_id().unwrap().to_string())
         
        }) {
        Ok(object_id) =>{
          Ok(ID::from(object_id))
        },
        Err(_mongodb_error) => Err(graphql_error::new("Server Error")),
    }
	}

	async fn delete_thread(&self, ctx: &Context<'_>,thread_id:String)->FieldResult<ThreadObject>{

		match ctx.data_unchecked::<crate::AppState>().container.support.delete_one(&thread_id).await
        .and_then(|document| {
			Ok(document.unwrap())
         
        }) {
        Ok(thread) => match thread {
			
            thread => match bson::from_document::<ThreadDeserializeObject>(thread) {
                Ok(thread) => {
				let thread=ThreadSerializeObject::build_from(thread);
				Ok(ThreadObject{
        				id: thread.id,
        				title: thread.title,
        				thread_description: thread.thread_description,
        				user_messages: thread.user_messages,
    			})
        
			}
                Err(_bson_de_error) => Err(graphql_error::new("No Threads Found")),
            },
        },
        Err(_mongodb_error) => Err(graphql_error::new("Server Error")),
    }
	}

	async fn send_msg(&self, ctx: &Context<'_>,thread_id:String,username:String ,msg:Option<String>,file: Option<FileInput>) -> FieldResult<ID> {
		
		let mut store = ctx.data_unchecked::<Storage>().lock().await;
		let entry = store.vacant_entry();
		let id: ID= entry.key().into();
		//note file saving in not async need to use async file write
		let file =match file {
			Some(file)=>{
				FileOutput{
					name:file.name,
					src:file.src
				}
			}, 
			None=>FileOutput{
				name:"".to_string(),
				src:"".to_string()
			}
		};
		let msg=match  msg {
			Some(msg)=>msg,
			None=>"".to_string()
		};
	
		let user =UserMessageObject{
			id:id.clone(),
			username:username,
			text:Some(msg),
			attachment:Some(file)
		};
		entry.insert(user.clone());
		match ctx.data_unchecked::<crate::AppState>().container.support.add_message(&thread_id,user.clone()).await
        .and_then(|document| {
            Ok(match document {
                Some(doc) => doc,
                None => bson::Document::new(),
            })
        }) {
        Ok(document) => match document {
			
            document => match bson::from_document::<ThreadDeserializeObject>(document) {
                Ok(thread) => {
					SimpleBroker::publish(StreamChanged {
						mutation_type: MutationType::Created,
						id: id.clone(),
					});
					Ok(id)
				// let thread=ThreadSerializeObject::build_from(thread);
				// Ok(thread.user_messages.unwrap())
                }
                Err(_bson_de_error) => Err(graphql_error::new("No Threads Found")),
            },
        },
        Err(_mongodb_error) => Err(graphql_error::new("Server Error")),
    }
		
	}

	// async fn remove_msg(&self, ctx: &Context<'_>,thread_id:String,message_id:ID) -> ID {

	// 	let mut store = ctx.data_unchecked::<Storage>().lock().await;
	// 	store.remove(message_id);
	// 	let entry = store.vacant_entry();
	// 	let id: ID= message_id;
		
		
	// 	match ctx.data_unchecked::<crate::AppState>().container.support.remove_message(&thread_id,user.clone()).await
    //     .and_then(|document| {
    //         Ok(match document {
    //             Some(doc) => doc,
    //             None => bson::Document::new(),
    //         })
    //     }) {
    //     Ok(document) => match document {
			
    //         document => match bson::from_document::<ThreadDeserializeObject>(document) {
    //             Ok(thread) => {
	// 				SimpleBroker::publish(StreamChanged {
	// 					mutation_type: MutationType::Created,
	// 					id: id.clone(),
	// 				});
	// 				Ok(id)
	// 			// let thread=ThreadSerializeObject::build_from(thread);
	// 			// Ok(thread.user_messages.unwrap())
    //             }
    //             Err(_bson_de_error) => Err(graphql_error::new("No Threads Found")),
    //         },
    //     },
    //     Err(_mongodb_error) => Err(graphql_error::new("Server Error")),
    // }
		// //note file saving in not async need to use async file write
		// let file =match file {
		// 	Some(file)=>{
		// 		FileOutput{
		// 			name:file.name,
		// 			src:file.src
		// 		}
		// 	}, 
		// 	None=>FileOutput{
		// 		name:"".to_string(),
		// 		src:"".to_string()
		// 	}
		// };
		// let msg=match  msg {
		// 	Some(msg)=>msg,
		// 	None=>"".to_string()
		// };
	
		// let user =UserMessageObject{
		// 	id:id.clone(),
		// 	username:username,
		// 	text:Some(msg),
		// 	attachment:Some(file)
		// };
		// entry.insert(user);
		// SimpleBroker::publish(StreamChanged {
		// 	mutation_type: MutationType::Created,
		// 	id: id.clone(),
		// });
		// id
	// }

	
}

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
enum MutationType {
	Created
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

	async fn user_message(&self, ctx: &Context<'_>) -> Result<Option<UserMessageObject>> {
		let message = ctx.data_unchecked::<Storage>().lock().await;
		let id = self.id.parse::<usize>()?;
		Ok(message.get(id).cloned())
	}
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
	

	async fn connect_stream(&self, mutation_type: Option<MutationType>) -> impl Stream<Item = StreamChanged> {
		SimpleBroker::<StreamChanged>::subscribe().filter(move |event| {
			let res = if let Some(mutation_type) = mutation_type {
				event.mutation_type == mutation_type
			} else {
				true
			};
			async move { res }
		})
	}
}

