#[macro_use]
extern crate log;

use actix_web::{guard, web, web::Data, App, HttpResponse, HttpServer, Responder};
use async_graphql::{
    http::GraphiQLSource, Context, EmptyMutation, EmptySubscription, MergedObject, Object,
    SimpleObject,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

struct AppData {
    schema: Schema,
    #[allow(dead_code)]
    db: PgPool,
}

#[derive(SimpleObject)]
struct User {
    id: u32,
    name: String,
    age: u32,
}
#[derive(Default)]
struct UserQuery;

#[Object]
impl UserQuery {
    async fn user(&self) -> User {
        todo!()
    }
}
#[derive(Default)]
struct RootQuery;

#[Object]
impl RootQuery {
    async fn healthz(&self, _ctx: &Context<'_>) -> async_graphql::Result<bool> {
        Ok(true)
    }
}

#[derive(MergedObject, Default)]
struct Query(RootQuery, UserQuery);

async fn handler(app_data: web::Data<AppData>, req: GraphQLRequest) -> GraphQLResponse {
    info!("{:?}", req.0);
    app_data.schema.execute(req.into_inner()).await.into()
}

async fn playground() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish())
}

fn build_db_url() -> String {
    let postgres_user = env::var("POSTGRES_USER").expect("POSTGRES_USER not set");
    let postgres_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not set");
    let postgres_db = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let postgres_host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST not set");

    format!(
        "postgres://{}:{}@{}/{}",
        postgres_user, postgres_password, postgres_host, postgres_db
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    let postgres_url = build_db_url();
    let port = env::var("PORT")
        .unwrap_or_else(|_| String::from("8080"))
        .parse::<u16>()
        .unwrap();

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await
    {
        Ok(pool) => {
            info!("Connected to database at {}", postgres_url);
            pool
        }
        Err(err) => {
            error!(
                "Failed to connect to database at {}: {:?}",
                postgres_url, err
            );
            std::process::exit(1)
        }
    };
    info!("playground started at: http://localhost:{}/graphql", port);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppData {
                schema: schema.clone(),
                db: pool.clone(),
            }))
            .service(web::resource("/graphql").guard(guard::Post()).to(handler))
            .service(web::resource("/graphql").guard(guard::Get()).to(playground))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
