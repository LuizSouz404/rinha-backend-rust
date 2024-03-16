use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use time::{macros::date, Date};

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let mut people: HashMap<Uuid, Person> = HashMap::new();

    let person = Person {
        id: Uuid::now_v7(),
        name: String::from("Luiz Souza"),
        nick: String::from("Souz"),
        birth_date: date!(2001 - 08 - 18),
        stack: vec!["Rust".to_string(), "Go".to_string()].into(),
    };

    println!("{}", person.id);

    people.insert(person.id, person);

    let app_state = Arc::new(RwLock::new(people));

    let app = Router::new()
        .route("/pessoas", get(search_people))
        .route("/pessoas/:id", get(find_people))
        .route("/pessoas", post(create_people))
        .route("/contagem-pessoas", get(count_people))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn search_people() -> impl IntoResponse { 
    // let State(people) = state;   
    (StatusCode::OK, "Busca pessoas");
}

async fn find_people(
    State(people): State<AppState>, 
    Path(person_id): Path<Uuid>
) -> impl IntoResponse {
    match people.read().await.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_people(
    State(people): State<AppState>, 
    Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
    let id = Uuid::now_v7();

    let person = Person {
        id,
        name: new_person.name,
        nick: new_person.nick,
        birth_date: new_person.birth_date,
        stack: new_person.stack
    };

    people.write().await.insert(id, person.clone());

    println!("{}", id);

    (StatusCode::OK, Json(person))
}

async fn count_people(
    State(people): State<AppState>, 
) -> impl IntoResponse {
    let count = people.read().await.len();
    (StatusCode::OK, Json(count))
}