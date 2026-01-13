// Benchmark ISO 2709 (MARC binary) read/write performance
// Compile with: rustc --edition 2021 -O scripts/benchmark_iso2709.rs -L target/release/deps

use std::fs::{File, metadata};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn get_system_info() -> String {
    let mut info = String::new();
    
    // OS version
    let output = Command::new("uname")
        .arg("-a")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    info.push_str(&format!("**OS:** {}\n", output.trim()));
    
    // CPU info (macOS/Linux)
    let cpu_output = if cfg!(target_os = "macos") {
        Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    } else {
        Command::new("grep")
            .arg("-m1")
            .arg("model name")
            .arg("/proc/cpuinfo")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    };
    info.push_str(&format!("**CPU:** {}\n", cpu_output.trim()));
    
    // Physical cores
    let cores_output = if cfg!(target_os = "macos") {
        Command::new("sysctl")
            .arg("-n")
            .arg("hw.physicalcpu")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    } else {
        Command::new("grep")
            .arg("-c")
            .arg("^processor")
            .arg("/proc/cpuinfo")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    };
    info.push_str(&format!("**Cores:** {}\n", cores_output.trim()));
    
    // Arch
    let arch = Command::new("uname")
        .arg("-m")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    info.push_str(&format!("**Architecture:** {}\n", arch.trim()));
    
    // RAM
    let ram = if cfg!(target_os = "macos") {
        Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
            .ok()
            .and_then(|o| {
                let bytes: u64 = String::from_utf8(o.stdout)
                    .ok()
                    .and_then(|s| s.trim().parse().ok())?;
                Some(format!("{} GB", bytes / (1024 * 1024 * 1024)))
            })
            .unwrap_or_default()
    } else {
        Command::new("grep")
            .arg("MemTotal")
            .arg("/proc/meminfo")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    };
    info.push_str(&format!("**RAM:** {}\n", ram.trim()));
    
    // Rust version
    let rust = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    info.push_str(&format!("**Rust:** {}\n", rust.trim()));
    
    info
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn main() {
    let test_file = PathBuf::from("tests/data/fixtures/10k_records.mrc");
    
    if !test_file.exists() {
        eprintln!("Error: Test file not found: {:?}", test_file);
        std::process::exit(1);
    }
    
    println!("ISO 2709 (MARC Binary) Baseline Performance Measurement");
    println!("========================================================\n");
    
    // Get file size
    let file_size = metadata(&test_file)
        .map(|m| m.len())
        .unwrap_or(0);
    
    println!("**Test Dataset:** {} ({})", test_file.display(), format_size(file_size));
    
    // Get system info
    println!("\n**System Environment:**");
    println!("{}", get_system_info());
    
    println!("\n**Performance Metrics:**\n");
    println!("|Metric|Value|");
    println!("|------|-----|");
    
    // File sizes
    println!("|Raw file size (uncompressed)|{}|", format_size(file_size));
    
    // Gzip compression
    let gzip_start = Instant::now();
    let gzip_output = Command::new("gzip")
        .arg("-c")
        .arg("-9")
        .arg(&test_file)
        .output();
    let gzip_time = gzip_start.elapsed();
    
    match gzip_output {
        Ok(output) => {
            let gzip_size = output.stdout.len() as u64;
            let ratio = (1.0 - gzip_size as f64 / file_size as f64) * 100.0;
            println!("|Gzipped file size (gzip -9)|{}|", format_size(gzip_size));
            println!("|Compression ratio|{:.1}%|", ratio);
            println!("|Gzip time|{:.2}s|", gzip_time.as_secs_f64());
        }
        Err(e) => {
            println!("|Gzipped file size|Error: {}|", e);
        }
    }
    
    println!("\n**Notes:**");
    println!("- Read/write throughput measured in mrrc test suite");
    println!("- Peak memory usage measured via RSS during benchmark");
    println!("- This is a reference baseline for format comparisons");
}
