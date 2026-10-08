/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub fn components(path: &str) -> Vec<&str> {
    path.split("/").filter(|s| !s.is_empty()).collect()
}

pub fn join(base: &str, name: &str) -> String {
    if base.is_empty() || base == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", base, name)
    }
}

pub fn parent_and_name(path: &str) -> (String, String) {
    let comps = components(path);

    if comps.is_empty() {
        return ("/".to_string(), String::new());
    }

    let name = comps[comps.len() - 1].to_string();
    let parent = if comps.len() == 1 {
        "/".to_string()
    } else {
        format!("/{}", comps[..comps.len() - 1].join("/"))
    };

    (parent, name)
}

pub fn normalize(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() { "/".to_string() } else { trimmed.to_string() }
}
