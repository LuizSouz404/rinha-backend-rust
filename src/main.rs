use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{de::value, Deserialize, Serialize};
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
    pub name: PersonName,
    #[serde(rename = "apelido")]
    pub nick: PersonNick,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<PersonTech>>,
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PersonName(String);
impl TryFrom<String> for PersonName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 100 {
            Ok(PersonName(value))
        } else {
            Err("name is too big")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PersonNick(String);
impl TryFrom<String> for PersonNick {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 32 {
            Ok(PersonNick(value))
        } else {
            Err("nick is too big")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PersonTech(String);
impl TryFrom<String> for PersonTech {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 32 {
            Ok(PersonTech(value))
        } else {
            Err("tech name is too big")
        }
    }
}

impl From<PersonTech> for String {
    fn from(value: PersonTech) -> Self {
        value.0
    }
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
    (StatusCode::OK, "Busca pessoas")
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
        name: new_person.name.0,
        nick: new_person.nick.0,
        birth_date: new_person.birth_date,
        stack: new_person.
            stack
            .map(|stack| stack.into_iter().map(String::from).collect()),
    };

    people.write().await.insert(id, person.clone());

    println!("{}", id);

    (StatusCode::CREATED, Json(person))
}

async fn count_people(
    State(people): State<AppState>, 
) -> impl IntoResponse {
    let count = people.read().await.len();
    (StatusCode::OK, Json(count))
}