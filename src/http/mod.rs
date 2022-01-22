// (c) Copyright 2021-2022 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::net::SocketAddr;

// extern usings
use hyper::{
    service::make_service_fn, service::service_fn, Body, Method, Request, Response, Server,
    StatusCode,
};
use prometheus::{Encoder, ProtobufEncoder, TextEncoder, PROTOBUF_FORMAT, TEXT_FORMAT};

async fn metrics(req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
    let mut buffer = vec![];

    let accepts_protobuf = req
        .headers()
        .get_all("accept")
        .iter()
        .any(|header| header == PROTOBUF_FORMAT);

    let metric_families = prometheus::gather();
    let content_type = if accepts_protobuf {
        let encoder = ProtobufEncoder::new();
        if encoder.encode(&metric_families, &mut buffer).is_err() {
            return server_error();
        }
        PROTOBUF_FORMAT
    } else {
        let encoder = TextEncoder::new();
        if encoder.encode(&metric_families, &mut buffer).is_err() {
            return server_error();
        }
        TEXT_FORMAT
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type)
        .body(Body::from(buffer))
}

async fn ready() -> Result<Response<Body>, hyper::http::Error> {
    no_content()
}

async fn live() -> Result<Response<Body>, hyper::http::Error> {
    no_content()
}

#[inline]
fn no_content() -> Result<Response<Body>, hyper::http::Error> {
    Response::builder()
        .status(hyper::StatusCode::NO_CONTENT)
        .body(Body::empty())
}

#[inline]
fn server_error() -> Result<Response<Body>, hyper::http::Error> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("Internal Server Error"))
}

#[inline]
fn not_found() -> Result<Response<Body>, hyper::http::Error> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
}

async fn router(req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => metrics(req).await,
        (&Method::GET, "/live") => live().await,
        (&Method::GET, "/ready") => ready().await,
        _ => not_found(),
    }
}

/// Listen for HTTP requests.
pub async fn listen(addr: &SocketAddr) -> Result<(), hyper::Error> {
    let svc = make_service_fn(|_| async { Ok::<_, hyper::http::Error>(service_fn(router)) });
    let srv = Server::bind(addr).serve(svc);
    srv.await?;
    Ok(())
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_not_found() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/nope")
            .body(Body::empty())
            .expect("failed to generate /nope request");

        let res = aw!(router(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_server_error() {
        let res = server_error();
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_no_content() {
        let res = no_content();
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_ready() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/ready")
            .body(Body::empty())
            .expect("failed to generate /live request");

        let res = aw!(router(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_live() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/live")
            .body(Body::empty())
            .expect("failed to generate /live request");

        let res = aw!(router(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_metrics_proto() {
        let req = Request::builder()
            .header("accept", PROTOBUF_FORMAT)
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .expect("failed to generate metrics request");

        let res = aw!(router(req));
        assert!(res.is_ok());
        let res = res.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[test]
    fn test_metrics_text() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .expect("failed to generate metrics request");

        let res = aw!(router(req));
        assert!(res.is_ok());
        let res = res.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }
}
