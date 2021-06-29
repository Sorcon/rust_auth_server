#[macro_use]
extern crate log;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use dotenv::dotenv;
use listenfd::ListenFd;
use sqlx::PgPool;
use std::env;

mod routes;

async fn index() -> impl Responder {
  HttpResponse::Ok().body(r#"
  Available routes:
  GET /users -> list of all users
  POST /users -> create new user
  GET /users/{id} -> get one user with requested id
  PUT /users/{id} -> update user with requested id
  DELETE /users/{id} -> delete user with requested id
  "#)
}


#[actix_web::main]
async fn main() -> Result<()> {
  dotenv().ok();
  env_logger::init();

  let mut listenfd = ListenFd::from_env();

  let database_url =
    env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
  let db_pool = PgPool::connect(&database_url).await?;

  let mut server = HttpServer::new(move || {
    App::new()
      .data(db_pool.clone())
      .route("/", web::get().to(index))
      .configure(routes::init_users)
  });
  let host = env::var("HOST").expect("HOST is not set in .env file");
  let port = env::var("PORT").expect("PORT is not set in .env file");
  let address = format!("{}:{}",host,port);

  server = match listenfd.take_tcp_listener(0)? {
    Some(listener) => server.listen(listener)?,
    None => {
      server.bind(address)?
    }
  };

  info!("Starting server on {}", address);
  server.run().await?;

  Ok(())
}
