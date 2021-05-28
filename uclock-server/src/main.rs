use uclock::Color;

use futures::stream::StreamExt;
use hyper::{
    header::HeaderValue,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use std::convert::Infallible;

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let color_str = req.uri().path().trim_start_matches("/");
    let color = match color_str.parse::<Color>() {
        Ok(color) => color,
        Err(_) if color_str == "" => Color::White,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(e))
                .unwrap())
        }
    };

    let stream = uclock::clock_stream(color).map(|x| -> Result<String, Infallible> { Ok(x) });

    let mut response = Response::new(Body::wrap_stream(stream));
    let headers = response.headers_mut();
    headers.append(
        hyper::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain"),
    );
    headers.append(
        hyper::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );
    headers.append("X-Accel-Buffering", HeaderValue::from_static("no"));
    Ok(response)
}

#[tokio::main]
async fn main() -> hyper::Result<()> {
    dotenv::dotenv().ok();

    let addr = std::env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
        .parse()
        .expect("Could not parse ADDR");

    let make_svc = make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(handle)) });
    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    server.await?;

    Ok(())
}
