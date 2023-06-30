use actix_web::{guard, web, web::Data, App, HttpResponse, HttpServer, Responder};
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Object, SimpleObject};
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

// struct Query;
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

async fn handler(app_data: web::Data<AppData>, req: GraphQLRequest) -> GraphQLResponse {
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
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
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
            println!("Connected to database at {}", postgres_url);
            pool
        }
        Err(err) => {
            println!(
                "Failed to connect to database at {}: {:?}",
                postgres_url, err
            );
            std::process::exit(1)
        }
    };
    println!("playground started at: http://localhost:8000/graphql");

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
