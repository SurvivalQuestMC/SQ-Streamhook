use std::{collections::HashMap, time::Duration};

use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use tokio::sync::mpsc::Sender;
use tower_http::timeout::TimeoutLayer;

pub async fn streamhook_server(state: String) -> String {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);
    let (shutdown_sender, mut shutdown_receiver) = tokio::sync::mpsc::channel::<()>(1);

    let app = Router::new()
        .route("/", get(authenticate_user))
        .with_state((state, tx, shutdown_sender))
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_receiver.recv().await;
        })
        .await
        .unwrap();
    let code = rx.recv().await.unwrap();
    code
}

async fn authenticate_user(
    State((request_state, tx, shutdown)): State<(String, Sender<String>, Sender<()>)>,
    Query(params): Query<HashMap<String, String>>,
) {
    println!("{:#?}", params);

    if params.contains_key("code") {
        if params.get("state") == Some(&request_state) {
            tx.send(params.get("code").unwrap().to_string())
                .await
                .unwrap();
            shutdown.send(()).await.unwrap();
        } else {
            println!("The State value is wrong!");
            println!("Expected Value: {}", request_state);
            println!("Received Value: {}", params.get("state").unwrap());
        }
    } else if params.contains_key("error") {
        println!("Error, User rejected auth, Please Try Again");
    } else {
        println!("Unexpected Result");
    }
}
