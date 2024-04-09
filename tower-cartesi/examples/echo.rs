use futures_util::FutureExt;
use std::env;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tower_cartesi::{listen_http, Request, Response};
use tower_service::Service;

type BoxError = Box<dyn Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let server_addr = env::var("ROLLUP_HTTP_SERVER_URL")?;

    let mut app = EchoApp {};
    tracing::info!("Listening on: {}", server_addr);
    listen_http(&mut app, &server_addr).await?;

    Ok(())
}

struct EchoApp;

impl Service<Request> for EchoApp {
    type Response = Response;
    type Error = Box<dyn Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        match req {
            Request::AdvanceState { metadata, payload } => {
                println!(
                    "Received advance state request {:?} \npayload {:?}:",
                    metadata, payload
                );
            }
            Request::InspectState { payload } => {
                println!("Received inspect state request {:?}", payload);
            }
        }
        async { Ok(tower_cartesi::Response::empty_accept()) }.boxed()
    }
}
