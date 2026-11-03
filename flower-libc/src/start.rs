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

use alloc::vec::Vec;
use core::arch::global_asm;
use core::ffi::{CStr, c_char};
use core::ptr;

use crate::{allocator, debugln, platform, process};

global_asm!(
    "
    .globl _start
    .type _start, @function
_start:
    mov rdi, rsp
    and rsp, 0xFFFFFFFFFFFFFFF0
    sub rsp, 8

    mov DWORD PTR [rsp], 0x00001F80
    ldmxcsr [rsp]
    mov WORD PTR [rsp], 0x037F
    fldcw [rsp]

    add rsp, 8

    call flowerlibc_crt0
    .size _start, . - _start
    "
);

#[repr(C)]
pub struct Stack {
    pub argc: isize,
    pub argv0: *const c_char,
}

impl Stack {
    pub fn argv(&self) -> *const *const c_char { ptr::from_ref(&self.argv0) }

    pub fn envp(&self) -> *const *const c_char {
        unsafe { self.argv().add(self.argc as usize + 1) }
    }

    pub fn auxv(&self) -> *const (usize, usize) {
        unsafe {
            let mut envp = self.envp();
            while !(*envp).is_null() {
                envp = envp.add(1);
            }
            envp.add(1).cast::<(usize, usize)>()
        }
    }
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn flowerlibc_crt0(sp: &'static Stack) -> ! {
    allocator::install();

    debugln!("flowerlibc_crt0: sp at {:p}", sp);
    debugln!("flowerlibc_crt0: argc = {}, argv0 = {:?}", sp.argc, sp.argv0);

    let args: Vec<&str> = {
        let argc = sp.argc;
        let argv = sp.argv();

        (0..argc)
            .map(|i| unsafe {
                CStr::from_ptr(*argv.offset(i)).to_str().unwrap()
            })
            .collect()
    };

    unsafe {
        platform::argv = args;
    }

    debugln!("flowerlibc_crt0: args = {:?}", unsafe { &platform::argv });

    unsafe {
        unsafe extern "C" {
            fn main() -> i32;
        }

        process::exit(main() as u64);
    }
}
