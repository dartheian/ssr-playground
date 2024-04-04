use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::ServeDir;

const ADD_URL: &str = "/add";

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    add_url: &'static str,
}

async fn home() -> HomeTemplate {
    HomeTemplate { add_url: ADD_URL }
}

#[derive(Template)]
#[template(path = "list.html")]
struct ListTemplate {
    names: Vec<String>,
}

#[derive(Deserialize)]
struct AddForm {
    name: String,
}

async fn add(State(state): State<Arc<Mutex<Vec<String>>>>, Form(form): Form<AddForm>) -> Response {
    if form.name.len() < 4 {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Name must be at least 4 character long",
        )
            .into_response()
    } else {
        let mut guard = state.lock().unwrap();
        guard.push(form.name);
        (
            StatusCode::OK,
            ListTemplate {
                names: guard.to_vec(),
            },
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(vec![]));
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(home))
        .route(ADD_URL, post(add))
        .with_state(state);
    let listener = TcpListener::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .await
        .unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async { signal::ctrl_c().await.unwrap() })
        .await
        .unwrap();
}
