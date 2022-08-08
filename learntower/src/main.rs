use std::{collections::HashMap, fmt::Debug, time::Duration, sync::{Arc, atomic::{AtomicUsize, Ordering}}, pin::Pin, future::Future, task::Poll};

use tower::{Service, ServiceExt};

pub struct Request {
    pub path_and_query: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Response {
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub async fn run<S>(mut service: S)
where
    S: Service<Request, Response = Response>,
    S::Error: Debug,
    S::Future: Send + 'static
{
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let request = Request {
            path_and_query: "/fake/path?page=1".to_owned(),
            headers: HashMap::new(),
            body: Vec::new(),
        };

        let service = match service.ready().await {
            Err(e) => {
                eprintln!("Service not able to accept requests: {:?}", e);
                continue;
            },
            Ok(service) => service,
        };

        let future = service.call(request);

        tokio::spawn(async move {
            match future.await {
                Ok(response) => println!("Successful response: {:?}", response),
                Err(e) => eprintln!("Error occurred: {:?}", e),
            }
        });
    }
}

#[derive(Default)]
pub struct DemoApp {
    counter: Arc<AtomicUsize>,
}

impl Service<Request> for DemoApp {
    type Response = Response;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let counter = self.counter.clone();

        Box::pin(async move {
            println!("Handling a request for {}", req.path_and_query);
            let counter = counter.fetch_add(1, Ordering::SeqCst);
            anyhow::ensure!(counter % 4 != 2, "Failing 25% of the time, just for fun");

            req.headers.insert("X-Counter".to_owned(), counter.to_string());

            let response = Response {
                status: 200,
                headers: req.headers,
                body: req.body,
            };

            Ok(response)
        })
    }
}

pub struct AppFn<F> {
    f: F,
}

pub fn app_fn<F, Ret>(f: F) -> AppFn<F>
where
    F: FnMut(Request) -> Ret,
    Ret: Future<Output = Result<Response, anyhow::Error>>,
{
    AppFn { f }
}

impl<F, Ret> Service<Request> for AppFn<F>
where
    F: FnMut(Request) -> Ret,
    Ret: Future<Output = Result<Response, anyhow::Error>>,
{
    type Response = Response;
    type Error = anyhow::Error;
    type Future = Ret;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        (self.f)(req)
    }
}

#[tokio::main]
async fn main() {
    let counter = Arc::new(AtomicUsize::new(0));

    run(app_fn(move |mut req| {
        let counter = Arc::clone(&counter);

        async move {
            println!("Handling request for {}", req.path_and_query);
            let counter = counter.fetch_add(1, Ordering::SeqCst);
            anyhow::ensure!(counter % 4 != 2, "Failing 25% of the time, just for fun");

            req.headers.insert("X-Counter".to_owned(), counter.to_string());

            let res = Response {
                status: 200,
                headers: req.headers,
                body: req.body,
            };

            Ok(res)
        }
    })).await;
}
