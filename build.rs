// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

const PROTO_DIR: &str = "./proto/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(&["./proto/kv.proto"], &[PROTO_DIR])?;

    Ok(())
}
