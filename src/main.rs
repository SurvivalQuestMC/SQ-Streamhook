use std::{collections::HashMap, net::SocketAddr};

use clap::Parser;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Method, Request, Response, StatusCode, body::Bytes, server::conn::http1, service::service_fn,
};
use hyper_util::rt::TokioIo;
use sq_streamhook::{
    StreamhookMessage,
    auth::{authenticate_user, refresh_streamhook},
    cli::{Cli, streamhook_parse_args},
    database::init_database,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match streamhook_parse_args(args) {
        StreamhookMessage::Stop => (),
        StreamhookMessage::Start => {
            streamhook_init().await?;

            streamhook_update()
        }
        StreamhookMessage::Debug => streamhook_listener().await?,
    }

    Ok(())
}

async fn listener_service(
    req: Request<hyper::body::Incoming>,
) -> anyhow::Result<Response<BoxBody<Bytes, hyper::Error>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let params: HashMap<String, String> = req
                .uri()
                .query()
                .map(|v| {
                    url::form_urlencoded::parse(v.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);
            for (key, value) in params {
                println!("{key}: {value}");
            }

            Ok(Response::new(full("Authenticated!")))
        }
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

async fn streamhook_listener() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(listener_service))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn streamhook_init() -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();
    let mut conn = init_database().await?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    refresh_streamhook(&mut conn, client).await?;
    authenticate_user()?;
    Ok(())
}

fn streamhook_update() {
    todo!();
    //loop {

    //}
}
