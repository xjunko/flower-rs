mod implementation;
mod types;

use core::arch::naked_asm;

use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::registers::control::{Efer, EferFlags};
use x86_64::registers::model_specific::{KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::arch::gdt;
use crate::system::syscalls::implementation::SYSCALL_HANDLERS;
use crate::system::syscalls::types::{SyscallError, SyscallFrame};
use crate::{debug, error};

#[repr(C, align(16))]
struct CPUContext {
    user: u64,
    kernel: u64,
}

static mut CPU_CONTEXT: CPUContext = CPUContext { user: 0, kernel: 0 };

#[allow(static_mut_refs)]
pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        CPU_CONTEXT.kernel = stack_top;
    }
}

#[allow(static_mut_refs)]
pub fn set_user_stack(stack_top: u64) {
    unsafe {
        CPU_CONTEXT.user = stack_top;
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
        // save user rsp to gs:[0], and load kernel rsp from gs:[8]
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

// syscall implementations
#[unsafe(no_mangle)]
extern "C" fn syscall_handler(frame: *mut SyscallFrame) {
    interrupts::without_interrupts(|| {
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
    })
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

    if let Some(handler) = SYSCALL_HANDLERS.get(num as usize).and_then(|h| *h) {
        match handler(arg1, arg2, arg3, arg4, arg5) {
            Ok(result) => result,
            Err(e) => {
                error!("syscall {} failed with error: {:?}", num, e);
                e as u64
            },
        }
    } else {
        error!("invalid syscall number: {}", num);
        SyscallError::NotFound as u64
    }
}
