//! Build script to compile Protocol Buffers and `FlatBuffers` schemas.
use std::io::Result;
use std::process::Command;

fn main() -> Result<()> {
    // Compile protobuf schema
    prost_build::compile_protos(&["proto/marc.proto"], &["proto"])?;

    // Regenerate FlatBuffers code if schema changes
    println!("cargo:rerun-if-changed=proto/marc.fbs");

    // Check if flatc is available and regenerate if so
    if let Ok(output) = Command::new("flatc").arg("--version").output() {
        if output.status.success() {
            let status = Command::new("flatc")
                .args(["--rust", "-o", "src/generated", "proto/marc.fbs"])
                .status()
                .expect("Failed to execute flatc");

            if !status.success() {
                println!("cargo:warning=flatc failed to generate FlatBuffers code");
            }
        }
    } else {
        println!("cargo:warning=flatc not found, skipping FlatBuffers code generation");
    }

    Ok(())
}
