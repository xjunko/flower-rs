use core::arch::naked_asm;

use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXIT, SYS_MSLEEP, SYS_OPEN, SYS_READ, SYS_WRITE,
};
use x86_64::VirtAddr;
use x86_64::registers::control::{Efer, EferFlags};
use x86_64::registers::model_specific::{KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::arch::gdt;
use crate::system::vfs::{FdKind, VFSError};
use crate::{debug, error, print, system};

#[repr(C, align(16))]
struct CPUStack {
    user: u64,
    kernel: u64,
}

static mut CPU_STACK: CPUStack = CPUStack { user: 0, kernel: 0 };

pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        CPU_STACK.kernel = stack_top;
    }
}

pub fn restore_kernel_gs_base() {
    let cpu_local = &raw const CPU_STACK as *const _ as u64;
    KernelGsBase::write(VirtAddr::new(cpu_local));
}

#[allow(static_mut_refs)]
pub fn install() {
    let segments = gdt::segments();

    unsafe {
        Star::write(
            segments.user_code,
            segments.user_data,
            segments.kernel_code,
            segments.kernel_data,
        )
        .expect("failed to write STAR");

        LStar::write(VirtAddr::new(syscall_entry as *const () as u64));
        SFMask::write(RFlags::empty());

        let efer = Efer::read();
        Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);

        let cpu_local = &raw const CPU_STACK as *const _ as u64;
        KernelGsBase::write(VirtAddr::new(cpu_local));
    }
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_entry() {
    naked_asm!(
        "cli",
        "swapgs",
        "mov gs:[0], rsp",
        "mov rsp, gs:[8]",
        //
        "push rcx",
        "push r11",
        "push rax",
        //
        "push rdi",
        "push rsi",
        "push rdx",
        "push r10",
        "push r8",
        "push r9",
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        //
        "mov rdi, rax",
        "mov rsi, [rsp + 11 * 8]",
        "mov rdx, [rsp + 10 * 8]",
        "mov rcx, [rsp + 9 * 8]",
        "mov r8, [rsp + 8 * 8]",
        "mov r9, [rsp + 7 * 8]",
        //
        "call {handler}",
        //
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "pop r9",
        "pop r8",
        "pop r10",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        //
        "add rsp, 8",
        "pop r11",
        "pop rcx",
        //
        "mov rsp, gs:[0]",
        "swapgs",
        //
        "sysretq",
        //
        handler = sym syscall_handler
    )
}

// syscall implementations

extern "C" fn syscall_handler(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> u64 {
    debug!(
        "syscall: num = {}, arg1 = {}, arg2 = {}, arg3 = {}, arg4 = {}, arg5 = {}",
        num, arg1, arg2, arg3, arg4, arg5
    );
    match num {
        SYS_EXIT => {
            system::proc::exit();
            unreachable!();
        },
        SYS_READ => {
            let fd = arg1 as usize;
            let buf = arg2 as *mut u8;
            let len = arg3 as usize;

            let result = system::proc::with_fd_table(|table| {
                match table.get_mut(fd)? {
                    FdKind::File(file) => {
                        let slice = unsafe {
                            core::slice::from_raw_parts_mut(buf, len)
                        };
                        file.read(slice)
                    },
                    _ => {
                        error!("read syscall: fd {} is not readable", fd);
                        Err(VFSError::PermissionDenied)
                    },
                }
            });

            result.unwrap_or(0) as u64
        },
        SYS_WRITE => {
            let fd = arg1 as usize;
            let buf = arg2 as *const u8;
            let len = arg3 as usize;

            let result =
                system::proc::with_fd_table(|table| match table.get(fd)? {
                    FdKind::Stdout | FdKind::Stderr => {
                        for i in 0..len {
                            let byte = unsafe { *buf.add(i) };
                            print!("{}", byte as char);
                        }
                        Ok(len)
                    },
                    _ => {
                        error!("write syscall: fd {} is not writable", fd);
                        Err(VFSError::PermissionDenied)
                    },
                });

            result.unwrap_or(0) as u64
        },
        SYS_OPEN => {
            let path_buf = arg1 as *const u8;
            let path_len = arg2 as usize;
            let flags = arg3 as u32;

            let path = unsafe {
                let slice = core::slice::from_raw_parts(path_buf, path_len);
                core::str::from_utf8_unchecked(slice)
            };

            // TODO: handle directory
            match system::vfs::open(path, flags) {
                Ok(file) => {
                    let result = system::proc::with_fd_table(|table| {
                        table.alloc(FdKind::File(file))
                    });
                    result.map(|fd| fd as u64).unwrap_or(u64::MAX)
                },
                Err(_) => u64::MAX,
            }
        },
        SYS_CLOSE => {
            let fd = arg1 as usize;
            let result = system::proc::with_fd_table(|table| table.close(fd));
            if result.is_ok() { 0 } else { u64::MAX }
        },
        SYS_MSLEEP => {
            let millis = arg1;
            system::proc::sleep(millis);
            0
        },
        _ => {
            error!("unknown syscall: {}", num);
            u64::MAX
        },
    }
}
