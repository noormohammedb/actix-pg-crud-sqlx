use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

use crate::{models::NoteModel, schema::FilterOptions, AppState};

#[get("/notes")]
pub async fn note_list_handler(
  opts: web::Query<FilterOptions>,
  state: web::Data<AppState>,
) -> impl Responder {
  let limit = opts.limit.unwrap_or(10);
  let offset = (opts.page.unwrap_or(1) - 1) * limit;

  let query_result = sqlx::query_as!(
    NoteModel,
    "SELECT * FROM notes ORDER BY id LIMIT $1 OFFSET $2",
    limit as i32,
    offset as i32
  )
  .fetch_all(&state.db)
  .await;

  if query_result.is_err() {
    let message = "Something bad happened while fetching all note items";
    return HttpResponse::InternalServerError().json(json!({"status":"error", "message": message}));
  }
  dbg!(&query_result);
  let notes = query_result.unwrap();

  let json_response = json!({
    "status": "success",
    "result": notes.len(),
    "notes": notes
  });

  HttpResponse::Ok().json(json_response)
}
