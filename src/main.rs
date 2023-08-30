use actix_web::{get, middleware::Logger, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde_json::json;

mod models;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  if std::env::var_os("RUST_LOG").is_none() {
    std::env::set_var("RUST_LOG", "actix_web=info");
  }
  env_logger::init();

  println!("server in started http://localhost:8080");

  HttpServer::new(move || {
    App::new()
      .service(health_checker_handler)
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
  // let json_data = json!({"foo": "bar"});
  dbg!(&json_data);
  HttpResponse::Ok().json(json_data)
}
