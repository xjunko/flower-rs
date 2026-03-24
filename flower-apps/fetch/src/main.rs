#![no_std]
#![no_main]

extern crate alloc;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;

use flower_libc::{println, std};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let (kernel_name, kernel_version) = {
        let kernelinfo_fd = std::open(b"/proc/version", 0, 0);
        if kernelinfo_fd < 0 {
            println!("failed to open /proc/version");
            std::exit(1);
        }
        let mut buf = vec![0u8; 1024];
        let len = std::read(kernelinfo_fd as u64, &mut buf);
        if len < 0 {
            println!("failed to read /proc/version");
            std::exit(1);
        }
        std::close(kernelinfo_fd as u64);

        let kernelinfo_str = String::from_utf8(buf)
            .unwrap_or_else(|_| "invalid data".to_string());

        // 0     1       2
        // Linux version 6.19.3-2-cachyos
        let parts: vec::Vec<&str> = kernelinfo_str.split_whitespace().collect();
        if parts.len() >= 3 {
            (parts[0].to_string(), parts[2].to_string())
        } else {
            ("unknown".to_string(), "unknown".to_string())
        }
    };

    let memory_info = {
        let meminfo_fd = std::open(b"/proc/meminfo", 0, 0);
        if meminfo_fd < 0 {
            println!("failed to open /proc/meminfo");
            std::exit(1);
        }
        let mut buf = vec![0u8; 1024];
        let len = std::read(meminfo_fd as u64, &mut buf);
        if len < 0 {
            println!("failed to read /proc/meminfo");
            std::exit(1);
        }
        std::close(meminfo_fd as u64);

        let meminfo_str = String::from_utf8(buf)
            .unwrap_or_else(|_| "invalid data".to_string());

        let mut map = BTreeMap::new();
        for line in meminfo_str.lines() {
            let parts: vec::Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<u64>() {
                    let key = parts[0].trim_end_matches(":").to_string();
                    map.insert(key, value);
                }
            }
        }
        map
    };

    println!(
        "flower@vocachuds
------------
Kernel: {} [v{}]
Memory: {}/{}MB",
        kernel_name,
        kernel_version,
        memory_info.get("MemAvailable").unwrap_or(&0) / 1024,
        memory_info.get("MemTotal").unwrap_or(&0) / 1024
    );

    std::exit(0);
}
