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

  println!("server is starting http://localhost:8080/");

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
    App::new()
      .app_data(
        web::Data::new(AppState { db: pool.clone() }), /* connect db and pass the connection */
      )
      .service(health_checker_handler)
      .service(handler::note_list_handler)
      .service(handler::create_note_handler)
      .service(handler::get_note_handler)
      .service(handler::edit_note_handler)
      .service(handler::delete_note_handler)
      .wrap(Logger::default())
  })
  .bind("0.0.0.0:8080")?
  .run()
  .await
}

#[get("/api/healthchecker")]
async fn health_checker_handler(_req: HttpRequest) -> impl Responder {
  const MESSAGE: &str = "foo bar koo";
  let json_data = json!({"status": "success", "message": MESSAGE});
  dbg!(&json_data);
  HttpResponse::Ok().json(json_data)
}
