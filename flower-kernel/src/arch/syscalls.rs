use core::arch::naked_asm;

use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXIT, SYS_MSLEEP, SYS_OPEN, SYS_READ, SYS_WRITE,
};
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::registers::control::{Efer, EferFlags};
use x86_64::registers::model_specific::{KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::arch::gdt;
use crate::system::vfs::{FdKind, VFSError};
use crate::{debug, error, print, system};

#[repr(C, align(16))]
struct CPUContext {
    user: u64,
    kernel: u64,
}

static mut CPU_CONTEXT: CPUContext = CPUContext { user: 0, kernel: 0 };
static mut SYSCALL_STACK: [u8; 4096 * 4] = [0; 4096 * 4];

#[allow(static_mut_refs)]
pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        let fallback_top = SYSCALL_STACK.as_ptr() as u64
            + core::mem::size_of_val(&SYSCALL_STACK) as u64;
        CPU_CONTEXT.kernel =
            if stack_top != 0 { stack_top } else { fallback_top };
    }
}

#[allow(static_mut_refs)]
pub fn write_cpu_context() {
    unsafe {
        let cpu_local = &CPU_CONTEXT as *const _ as u64;
        KernelGsBase::write(VirtAddr::new(cpu_local));
    }
}

#[allow(static_mut_refs)]
pub fn install() {
    interrupts::without_interrupts(|| {
        let segments = gdt::segments();

        unsafe {
            set_kernel_stack(0);

            // enable syscall
            let efer = Efer::read();
            Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);

            // disable interrupts during syscall
            SFMask::write(RFlags::INTERRUPT_FLAG);

            // set segments
            Star::write(
                segments.user_code,
                segments.user_data,
                segments.kernel_code,
                segments.kernel_data,
            )
            .expect("failed to write STAR");

            // entry point
            LStar::write(VirtAddr::new(syscall_entry as *const () as u64));

            let cpu_local = &CPU_CONTEXT as *const _ as u64;
            KernelGsBase::write(VirtAddr::new(cpu_local));
        }
    })
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_entry() {
    naked_asm!(
        "swapgs",
        // save user rsp to gs:[8], and load kernel rsp from gs:[0]
        "mov gs:[0], rsp",
        "mov rsp, gs:[8]",
        // top half of iret frame (ss, rsp, rflags, cs, rip)
        "push {user_ss}",
        "push gs:[0]",
        "push r11",
        "push {user_cs}",
        "push rcx",
        // bottom half
        "push rax",
        "push rcx",
        "push rdx",
        "push rbx",
        "push rbp",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        //
        "mov rdi, rsp",
        //
        "call {handler}",
        //
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rbp",
        "pop rbx",
        "pop rdx",
        "pop rcx",
        "pop rax",
        // done
        "swapgs",
        "iretq",
        //
        handler = sym syscall_handler,
        user_ss = const 0x1b, // hardcoded
        user_cs = const 0x23, // hardcoded
    )
}

#[repr(C)]
pub struct CPUFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: isize,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

// syscall implementations

#[unsafe(no_mangle)]
extern "C" fn syscall_handler(frame: *mut CPUFrame) {
    let frame = unsafe { &mut *frame };
    let result = syscall_handler_unwrapped(
        frame.rax as u64,
        frame.rdi,
        frame.rsi,
        frame.rdx,
        frame.r10,
        frame.r8,
    );
    frame.rax = result as isize;
}

fn syscall_handler_unwrapped(
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
