// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

#[tokio::main]
async fn main() {
    let code = librift::riftd::run().await;
    std::process::exit(code)
}