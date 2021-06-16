use bson::{doc, oid::ObjectId, Document};
use mongodb::{Collection, Cursor, error::Error, options::{DeleteOptions, FindOneAndDeleteOptions, FindOneAndDeleteOptionsBuilder, FindOneAndUpdateOptions, ReturnDocument}, results::InsertOneResult};

#[derive(Debug, Clone)]
pub struct SupportCollection {
    collection: Collection,
}

impl SupportCollection {
    pub fn new(collection: Collection) -> SupportCollection {
        SupportCollection { collection }
    }

    pub async fn find_one<T>(&self, document: T) -> Result<Option<Document>, Error>
    where
        T: serde::Serialize,
    {
        Ok(self
            .collection
            .find_one(
                bson::to_bson(&document)
                    .unwrap()
                    .as_document()
                    .unwrap()
                    .clone(),
                None,
            )
            .await?)
    }

    // pub async fn find_all(&self) -> Result<Cursor, Error> {
    //     Ok(self.collection.find(None, None).await?)
    // }

    pub async fn find_all_threads_for_project(&self,client_id:&str) -> Result<Cursor, Error> {
        Ok(self
            .collection
            .find(
                doc! {
                    "project_id":ObjectId::with_string(client_id).unwrap()
                },
                None,
            )
            .await?)
    }

    pub async fn find_thread_by_id(&self,thread_id:&str) -> Result<Cursor, Error> {
        Ok(self
            .collection
            .find(
                doc! {
                    "_id":ObjectId::with_string(thread_id).unwrap()
                },
                None,
            )
            .await?)
    }

    pub async fn insert_one<T>(&self, document: T) -> Result<InsertOneResult, Error>
    where
        T: serde::Serialize,
    {
        Ok(self
            .collection
            .insert_one(
                bson::to_bson(&document)
                    .unwrap()
                    .as_document()
                    .unwrap()
                    .clone(),
                None,
            )
            .await?)
    }

    pub async fn delete_one(&self, thread_id: &str) -> Result<Option<Document>, Error>
  
    {
        Ok(self
            .collection
            .find_one_and_delete(
                doc!{
                    "_id":thread_id
                }
                ,
      None,
            )
            .await?)
    }

 
    // pub async fn update_one<T>(&self, thread_id: &str, document: T) -> Result<Option<Document>, Error>
    // where
    //     T: serde::Serialize,
    // {
    //     Ok(self
    //         .collection
    //         .find_one_and_update(
    //             doc! {
    //                 "_id":ObjectId::with_string(thread_id).unwrap()
    //             },
    //             doc! {
    //                   "$set":bson::to_bson(&document)
    //                     .unwrap()
    //                     .as_document()
    //                     .unwrap()
    //                     .clone()

    //             },
    //             Some(
    //                 FindOneAndUpdateOptions::builder()
    //                     .return_document(ReturnDocument::After)
    //                     .build(),
    //             ),
    //         )
    //         .await?)
    // }
  

    

    pub async fn find_one_by_id(&self, thread_id: &str) -> Result<Option<Document>, Error> {
        Ok(self
            .collection
            .find_one(
                doc! {
                    "_id":ObjectId::with_string(thread_id).unwrap()
                },
                None,
            )
            .await?)
    }

    pub async fn add_message<T>(
        &self,
        thread_id: &str,
        document:T,
    ) -> Result<Option<Document>, Error>
    where T:serde::Serialize
    {
        Ok(self
            .collection
            .find_one_and_update(
                doc! {
                    "_id":ObjectId::with_string(thread_id).unwrap()
                },
                doc! {
                  "$push":{
                    "user_messages":bson::to_bson(&document)
                    .unwrap()
                    .as_document()
                    .unwrap()
                    .clone(),     
                  }
                },
                Some(
                    FindOneAndUpdateOptions::builder()
                        .return_document(ReturnDocument::After)
                        .build(),
                ),
            )
            .await?)
    }

    pub async fn remove_message<T>(
        &self,
        feautre_id: &str,
        document: T,
    ) -> Result<Option<Document>, Error>
    where
        T: serde::Serialize,
    {
        Ok(self
            .collection
            .find_one_and_update(
                doc! {
                    "_id":ObjectId::with_string(feautre_id).unwrap()
                },
                doc! {
                  "$pull":{
                      "user_messages":bson::to_bson(&document)
                      .unwrap()
                      .as_document()
                      .unwrap()
                      .clone(),
                  }
                },
                Some(
                    FindOneAndUpdateOptions::builder()
                        .return_document(ReturnDocument::After)
                        .build(),
                ),
            )
            .await?)
    }




}


  

    

   

