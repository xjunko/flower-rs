#![no_std]
#![no_main]

extern crate alloc;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;

use flower_libc::file::File;
use flower_libc::{println, process};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let (kernel_name, kernel_version) = {
        if let Ok(file) = File::open("/proc/version".to_string()) {
            let mut buf = vec![0u8; 1024];
            if let Err(e) = file.read(&mut buf) {
                println!("failed to read /proc/version: {}", e);
                process::exit(1);
            }

            let info_str = String::from_utf8(buf)
                .unwrap_or_else(|_| "invalid data".to_string());

            // 0     1       2
            // Linux version 6.19.3-2-cachyos
            let parts: vec::Vec<&str> = info_str.split_whitespace().collect();
            if parts.len() >= 3 {
                (parts[0].to_string(), parts[2].to_string())
            } else {
                ("unknown".to_string(), "unknown".to_string())
            }
        } else {
            println!("failed to open /proc/version");
            process::exit(1);
        }
    };

    let memory_info = {
        if let Ok(file) = File::open("/proc/meminfo".to_string()) {
            let mut buf = vec![0u8; 1024];
            if let Err(e) = file.read(&mut buf) {
                println!("failed to read /proc/meminfo: {}", e);
                process::exit(1);
            }

            let info_str = String::from_utf8(buf)
                .unwrap_or_else(|_| "invalid data".to_string());

            let mut map = BTreeMap::new();
            for line in info_str.lines() {
                let parts: vec::Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2
                    && let Ok(value) = parts[1].parse::<u64>()
                {
                    let key = parts[0].trim_end_matches(":").to_string();
                    map.insert(key, value);
                }
            }
            map
        } else {
            println!("failed to open /proc/meminfo");
            process::exit(1);
        }
    };

    let mem_total_kb = *memory_info.get("MemTotal").unwrap_or(&0);
    let mem_used_kb = if let Some(mem_used) = memory_info.get("MemUsed") {
        *mem_used
    } else {
        let mem_free_kb = if let Some(mem_free) = memory_info.get("MemFree") {
            *mem_free
        } else {
            *memory_info.get("MemAvailable").unwrap_or(&0)
        };
        mem_total_kb.saturating_sub(mem_free_kb)
    };

    println!(
        "flower@vocachuds
------------
Kernel: {} [v{}]
Memory: {}/{}MB",
        kernel_name,
        kernel_version,
        mem_used_kb / 1024,
        mem_total_kb / 1024
    );

    process::exit(0);
}
