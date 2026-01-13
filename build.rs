//! Build script to compile Protocol Buffers schema.
use std::io::Result;

fn main() -> Result<()> {
    // Compile protobuf schema
    prost_build::compile_protos(&["proto/marc.proto"], &["proto"])?;
    Ok(())
}
