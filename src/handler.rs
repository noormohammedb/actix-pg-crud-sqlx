use actix_web::{get, patch, post, web, HttpResponse, Responder};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::{
  models::NoteModel,
  schema::{CreateNoteSchema, FilterOptions, UpdatedNoteSchema},
  AppState,
};

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

#[post("/notes")]
pub async fn create_note_handler(
  body: web::Json<CreateNoteSchema>,
  state: web::Data<AppState>,
) -> impl Responder {
  dbg!(&body);
  dbg!(&state);

  let query_result = sqlx::query_as!(
    NoteModel,
    "INSERT INTO notes (title, content, category, published) VALUES($1, $2, $3, $4) RETURNING *",
    body.title,
    body.content,
    body.category,
    body.published,
  )
  .fetch_one(&state.db)
  .await;

  match query_result {
    Ok(note) => {
      let note_response = json!({
        "status": "success",
        "data": json!({
          "note": json!({"data": note})
        })
      });

      return HttpResponse::Ok().json(note_response);
    }
    Err(e) => {
      dbg!(&e);
      if e
        .to_string()
        .contains("duplicate key value violates unique constraint")
      {
        return HttpResponse::BadRequest().json(json!({
          "status": "fail",
          "message": "Note with same title exists"
        }));
      }
      return HttpResponse::InternalServerError().json(json!({
          "status": "error",
          "message": format!("{:?}", e)
      }));
      //
    }
  }
}

#[get("/notes/{id}")]
pub async fn get_note_handler(state: web::Data<AppState>, data: web::Path<Uuid>) -> impl Responder {
  dbg!(&state, &data);
  let note_id = data.into_inner();

  let query_result = sqlx::query_as!(NoteModel, "SELECT * FROM notes WHERE id = $1", note_id)
    .fetch_one(&state.db)
    .await;

  match query_result {
    Ok(note) => {
      let res_data = json!({
        "status": "success",
        "data": json!({
          "note": json!({"data": note})
        })
      });

      return HttpResponse::Ok().json(res_data);
    }
    Err(e) => {
      dbg!(&e);
      let message = format!("Note with ID: {} not found", note_id);
      return HttpResponse::NotFound().json(json!({
        "status": "fail",
        "message": message,
        "data": json!({ })
      }));
    }
  }
}

#[patch("/notes/{id}")]
pub async fn edit_note_handler(
  state: web::Data<AppState>,
  path: web::Path<Uuid>,
  body: web::Json<UpdatedNoteSchema>,
) -> impl Responder {
  let note_id = path.into_inner();

  let query_result = sqlx::query_as!(NoteModel, "SELECT * FROM notes WHERE ID = $1", note_id)
    .fetch_one(&state.db)
    .await;

  if query_result.is_err() {
    dbg!(&query_result.err());
    let message = format!("Note with ID: {} not found", note_id);
    return HttpResponse::NotFound().json(json!({"status": "Fail", "message": message,}));
  }

  let now = Utc::now();
  let note = query_result.unwrap();

  let query_result = sqlx::query_as!(
    NoteModel,
    "UPDATE notes set title = $1, content = $2, category = $3, published = $4, updated_at = $5 where id = $6 RETURNING *",
    body.title.to_owned().unwrap_or(note.title),
    body.content.to_owned().unwrap_or(note.content),
    body.category.to_owned().unwrap_or(note.category.unwrap()),
    body.published.to_owned().unwrap_or(note.published.unwrap()),
    now,
    note_id
  ).fetch_one(&state.db).await;

  match query_result {
    Ok(data) => {
      return HttpResponse::Ok().json(json!({
        "status": "success",
        "data": json!({
          "note": data
        })
      }));
    }
    Err(err) => {
      dbg!(&err);
      return HttpResponse::BadRequest().json(json!({
        "status": "fail",
        "message": format!("Error: {:?}", err)
      }));
    }
  }
}
