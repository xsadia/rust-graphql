use actix_web::{guard, web, web::Data, App, HttpResponse, HttpServer, Responder};
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Object, SimpleObject};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

#[derive(SimpleObject)]
struct User {
    id: u32,
    name: String,
    age: u32,
}

struct Query;
#[Object]
impl Query {
    async fn user(&self) -> User {
        User {
            id: 1,
            name: String::from("Felipe"),
            age: 25,
        }
    }
}

async fn handler(schema: web::Data<Schema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn playground() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    println!("playground started at: http://localhost:8000/graphql");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/graphql").guard(guard::Post()).to(handler))
            .service(web::resource("/graphql").guard(guard::Get()).to(playground))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
