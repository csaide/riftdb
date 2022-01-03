// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::net::SocketAddr;

use crate::grpc::pubsub;
use crate::grpc::subscription;
use crate::grpc::topic;
use crate::http;
use crate::log;
use crate::metric;
use crate::pubsub::Registry;

use exitcode::ExitCode;
use structopt::clap::{self, crate_version, ErrorKind};
use structopt::StructOpt;
use tonic::transport::Server;

const RIFTD: &str = "riftd";

/// Overall riftd binary configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    global_settings = &[clap::AppSettings::DeriveDisplayOrder],
    author = "Christian Saide <me@csaide.dev>",
    about = "Run an instance of riftd.",
    version = crate_version!()
)]
struct RiftdConfig {
    #[structopt(flatten)]
    log_config: log::Config,
    #[structopt(
        long = "grpc-addr",
        short = "g",
        env = "RIFT_GRPC_ADDR",
        help = "The address to listen on for incoming gRPC requests.",
        long_help = "This sets the listen address for all incoming gRPC requests.",
        default_value = "[::]:8081",
        takes_value = true
    )]
    grpc_addr: SocketAddr,
    #[structopt(
        long = "http-addr",
        short = "a",
        env = "RIFT_HTTP_ADDR",
        help = "The address to listen on for incoming HTTP requests.",
        long_help = "This sets the listen address for all incoming HTTP requests.",
        default_value = "[::]:8080",
        takes_value = true
    )]
    http_addr: SocketAddr,
}

/// Execute riftd.
pub async fn run() -> ExitCode {
    let setup_logger = log::default(RIFTD, crate_version!());
    let cfg = match RiftdConfig::from_args_safe() {
        Ok(cfg) => cfg,
        Err(err)
            if err.kind == ErrorKind::HelpDisplayed || err.kind == ErrorKind::VersionDisplayed =>
        {
            println!("{}", err.message);
            return exitcode::USAGE;
        }
        Err(err) => {
            crit!(setup_logger, "Failed to parse provided configuration."; "error" => err.to_string());
            return exitcode::CONFIG;
        }
    };

    let root_logger = log::new(&cfg.log_config, RIFTD, crate_version!());

    let mm = metric::Manager::new(
        "rift".to_string(),
        "grpc".to_string(),
        "riftd".to_string(),
        crate_version!().to_string(),
    );

    let registry = Registry::default();
    let pubsub_impl = pubsub::Handler::with_registry(registry.clone());
    let topic_impl = topic::Handler::with_registry(registry.clone());
    let sub_impl = subscription::Handler::with_registry(registry.clone());

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;
    health_reporter
        .set_service_status("pubsub", tonic_health::ServingStatus::Serving)
        .await;

    let grpc_logger = root_logger.new(o!("mod" => "grpc"));
    let grpc_handle = async move {
        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(topic::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(pubsub::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(subscription::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(
                tonic_health::proto::GRPC_HEALTH_V1_FILE_DESCRIPTOR_SET,
            )
            .build()
            .unwrap();
        let interceptor = crate::grpc::interceptor::RiftInterceptor::new(&grpc_logger, mm);

        info!(&grpc_logger, "Listening for gRPC requests."; "addr" => cfg.grpc_addr.to_string());
        if let Err(err) = Server::builder()
            .add_service(topic::TopicServiceServer::with_interceptor(
                topic_impl,
                interceptor.clone(),
            ))
            .add_service(pubsub::PubSubServiceServer::with_interceptor(
                pubsub_impl,
                interceptor.clone(),
            ))
            .add_service(subscription::SubscriptionServiceServer::with_interceptor(
                sub_impl,
                interceptor.clone(),
            ))
            .add_service(reflection)
            .add_service(health_service)
            .serve(cfg.grpc_addr)
            .await
        {
            crit!(&grpc_logger, "Failed to listen and serve gRPC."; "error" => err.to_string());
        }
    };

    let http_logger = root_logger.new(o!("mod" => "http"));
    let http_handle = async move {
        info!(&http_logger, "Listening for HTTP requests."; "addr" => cfg.http_addr.to_string());
        if let Err(err) = http::listen(&cfg.http_addr).await {
            crit!(&http_logger, "Failed to listen and serve HTTP."; "error" => err.to_string());
        }
    };

    info!(&root_logger, "Fully initialized and listening!");
    tokio::select! {
        _ = grpc_handle => {},
        _ = http_handle => {},
    };

    exitcode::IOERR
}
