use actix_cors::Cors;
use actix_web::{
  get, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod handler;
mod models;
mod schema;

#[derive(Debug)]
pub struct AppState {
  db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  if std::env::var_os("RUST_LOG").is_none() {
    std::env::set_var("RUST_LOG", "actix_web=info");
  }
  dotenv::dotenv().ok();
  env_logger::init();

  println!("server is starting http://localhost:8000/");

  let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  let pool = match PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await
  {
    Ok(pool) => {
      println!("connection to database is sucessfull");
      dbg!(&pool);
      pool
    }
    Err(err) => {
      println!("failed to connect to the database:\n{:#?}", err);
      std::process::exit(1)
    }
  };

  HttpServer::new(move || {
    let cors = Cors::default()
      .allow_any_origin()
      .allow_any_method()
      .allow_any_header();
    App::new()
      .app_data(web::Data::new(AppState { db: pool.clone() }))
      .service(health_checker_handler)
      .configure(handler::handler_service_config)
      .wrap(cors)
      .wrap(Logger::default())
  })
  .bind("0.0.0.0:8000")?
  .run()
  .await
}

#[get("/api/healthchecker")]
async fn health_checker_handler(_req: HttpRequest) -> impl Responder {
  const MESSAGE: &str = "foo bar koo";
  let json_data = json!({"status": "success", "message": MESSAGE});
  HttpResponse::Ok().json(json_data)
}
