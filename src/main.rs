use actix_web::http::header::ContentType;
use actix_web::{guard, web::{self,Data}, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig, MultipartOptions};
use async_graphql::Schema;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use mongodb::{options::ClientOptions, Client, Collection};
mod controllers;
use actix_cors::Cors;
use controllers::{GraphQlSchema, MutationRoot, MyToken, QueryRoot, SubscriptionRoot};
mod models;
use load_dotenv::load_dotenv;
use models::{Support, SupportCollection};
use std::env;

use crate::controllers::Storage;

async fn index(
    schema: web::Data<GraphQlSchema>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().map(|s| MyToken(s.to_string())).ok());

    let mut request = gql_request.into_inner();
    if let Some(token) = token {
        request = request.data(token);
    }
    schema.execute(request).await.into()
}
async fn index_ws(
    schema: web::Data<GraphQlSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

pub struct CollectionContainer {
    #[allow(dead_code)]
    support: SupportCollection,
}
impl CollectionContainer {
    pub fn new(support: SupportCollection) -> CollectionContainer {
        CollectionContainer { support }
    }
}

pub struct AppState {
    #[allow(dead_code)]
    container: CollectionContainer,
}

async fn establish_connection() -> Collection<Support> {
    load_dotenv!();
    let client_options = ClientOptions::parse(env!("DATABASE_URL")).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database(env!("SUPPORT_DATABASE"));
    db.collection(env!("SUPPORT_COLLECTION"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let support_collection = establish_connection().await;
    let collection_container =
        CollectionContainer::new(SupportCollection::new(support_collection.clone()));
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(Storage::default())
        .data(AppState {
            container: collection_container,
        })
        .finish();
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .expect("PORT must be a number");
    println!("Playground: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method()
                    .supports_credentials(),
            )
            .app_data(Data::new(schema.clone()))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(index)
                    .app_data(MultipartOptions::default().max_num_files(3)),
            )
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind(("0.0.0.0".to_string(), port))?
    .run()
    .await
}
