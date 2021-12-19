// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::net::SocketAddr;

// extern usings
use hyper::{
    service::make_service_fn, service::service_fn, Body, Method, Request, Response, Server,
    StatusCode,
};
use prometheus::{Encoder, TextEncoder};

async fn metrics() -> Result<Response<Body>, hyper::http::Error> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => Ok(Response::new(Body::from(buffer))),
        Err(_) => server_error(),
    }
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
        (&Method::GET, "/metrics") => metrics().await,
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
