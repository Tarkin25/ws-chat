use std::{convert::Infallible, net::SocketAddr, sync::{Arc, atomic::{AtomicUsize, Ordering}}, future::Ready, task::Poll};

use hyper::{Request, Body, Response, Server, service::Service, server::conn::AddrStream};

struct DemoApp {
    counter: Arc<AtomicUsize>,
}

impl Service<Request<Body>> for DemoApp {
    type Response = Response<Body>;
    type Error = hyper::http::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let res = Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(format!("Counter is at: {}", counter).into());

        std::future::ready(res)
    }
}

struct DemoAppFactory {
    counter: Arc<AtomicUsize>,
}

impl Service<&AddrStream> for DemoAppFactory {
    type Response = DemoApp;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, conn: &AddrStream) -> Self::Future {
        println!("Accepting a new connection from {:?}", conn);
        std::future::ready(Ok(DemoApp {
            counter: Arc::clone(&self.counter)
        }))
    }
}

#[tokio::main]
async fn main() {
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    let factory = DemoAppFactory {
        counter: Arc::new(AtomicUsize::new(0))
    };

    let server = Server::bind(&address).serve(factory);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
