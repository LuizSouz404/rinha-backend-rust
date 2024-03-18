use std::{env, sync::Arc};

use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use persistence::PostgresRepository;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::Date;

mod persistence;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize, sqlx::FromRow)]
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

type AppState = Arc<PostgresRepository>;

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://rinha:rinha@localhost:5432/rinha"));

    let repo = PostgresRepository::connect(database_url).await;

    let app_state = Arc::new(repo);

    let app = Router::new()
        .route("/pessoas", get(search_people))
        .route("/pessoas/:id", get(find_people))
        .route("/pessoas", post(create_people))
        .route("/contagem-pessoas", get(count_people))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct PersonSearchQuery {
    #[serde(rename = "t")]
    query: String,
}
async fn search_people(
    State(people): State<AppState>,
    Query(PersonSearchQuery { query }): Query<PersonSearchQuery>
) -> impl IntoResponse {
    match people.search_people(query).await {
        Ok(people) => Ok(Json(people)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn find_people(
    State(people): State<AppState>, 
    Path(person_id): Path<Uuid>
) -> impl IntoResponse {
    match people.find_person(person_id).await {
        Ok(Some(person)) => Ok(Json(person.clone())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_people(
    State(people): State<AppState>, 
    Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
    match people.create_person(new_person).await {
        Ok(person) => Ok((StatusCode::CREATED, Json(person.clone()))),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(StatusCode::UNPROCESSABLE_ENTITY)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn count_people(
    State(people): State<AppState>, 
) -> impl IntoResponse {
    match people.count_people().await {
        Ok(count) => Ok(Json(count)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}