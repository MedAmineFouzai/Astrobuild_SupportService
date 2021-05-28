use async_graphql::{ Object,ID,InputObject};
use bson::oid::ObjectId;
use serde::{Deserialize,Serialize};

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct FileOutput {
    pub name :String,
    pub src:String,
}

#[derive(Debug,InputObject,Clone,Deserialize,Serialize)]
pub struct FileInput {
    pub name :String,
    pub src:String,
}


#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct UserMessageObject {
    pub id:ID,
    pub username:String,
    pub text:Option<String>,
    pub attachment:Option<FileOutput>,

}

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct ThreadObject {
    pub id:String,
    pub title:String,
    pub thread_description:String,
    pub user_messages:Option<Vec<UserMessageObject>>,

}

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct Thread {
    pub project_id:ObjectId,
    pub title:String,
    pub thread_description:String,
    pub user_messages:Option<Vec<UserMessageObject>>,

}
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct ThreadSerializeObject {
    pub id:String,
    pub project_id:String,
    pub title:String,
    pub thread_description:String,
    pub user_messages:Option<Vec<UserMessageObject>>,

}

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct ThreadDeserializeObject {
    pub _id:ObjectId,
    pub project_id:ObjectId,
    pub title:String,
    pub thread_description:String,
    pub user_messages:Option<Vec<UserMessageObject>>,
}

impl ThreadSerializeObject {
     pub fn build_from(thread:ThreadDeserializeObject)->ThreadSerializeObject{
        ThreadSerializeObject{
            id:thread._id.to_string(),
            project_id:thread.project_id.to_string(),
            title:thread.title,
            thread_description:thread.thread_description,
            user_messages:thread.user_messages,
        }
    }
}

#[Object]
impl ThreadObject {
    async fn id(&self) -> &str {
        &self.id
    }
    async fn title(&self) -> &str {
        &self.title
    }
    async fn thread_description(&self) -> &str {
        &self.thread_description
    }
    async fn user_messages(&self) -> &Vec<UserMessageObject> {
        &self.user_messages.as_ref().unwrap()
    }

}

#[Object]
impl UserMessageObject {

    async fn id(&self) -> &str {
        &self.id
    }

    async fn username(&self) -> &str {
        &self.username
    }

    async fn text(&self) -> &str {
        &self.text.as_ref().unwrap()
    }

    async fn attachment(&self) -> &FileOutput {
       &self.attachment.as_ref().unwrap()
    }

}

#[Object]
impl FileOutput {
    async fn name(&self) -> &str {
        &self.name
    }
    
    async fn src(&self) -> &str {
        &self.src
    }

}



