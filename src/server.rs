use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Method, Request, Response, StatusCode, body::Bytes, server::conn::http1, service::service_fn,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

pub async fn receive_connection(
    f: fn(Request<hyper::body::Incoming>, Arc<Mutex<String>>) -> String,
    state_original: Arc<Mutex<String>>,
) -> anyhow::Result<()> {
    let port = 3000;
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse()?;
    let listener = TcpListener::bind(addr).await?;
    let state_value = String::from(state_original.lock().unwrap().as_str());
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state_original.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| listener_service(req, f, state.clone())),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });

        if state_value != *state_original.lock().unwrap() {
            break;
        }
    }

    tokio::select! {
        _ = graceful.shutdown() => {
            eprintln!("all connections gracefully closed");
            Ok(())
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            eprintln!("timed out wait for all connections to close");
            Ok(())
        }
    }
}

async fn listener_service(
    req: Request<hyper::body::Incoming>,
    f: fn(Request<hyper::body::Incoming>, Arc<Mutex<String>>) -> String,
    state: Arc<Mutex<String>>,
) -> anyhow::Result<Response<BoxBody<Bytes, hyper::Error>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full(f(req, state)))),
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
