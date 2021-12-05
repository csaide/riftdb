// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/example.proto")?;
    Ok(())
}
