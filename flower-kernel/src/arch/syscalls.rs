use core::arch::naked_asm;

use x86_64::VirtAddr;
use x86_64::registers::control::{Efer, EferFlags};
use x86_64::registers::model_specific::{KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::arch::gdt;
use crate::{debug, error, print, system};

#[repr(C, align(16))]
struct CPUStack {
    user: u64,
    kernel: u64,
}

static mut CPU_STACK: CPUStack = CPUStack { user: 0, kernel: 0 };
static mut SYSCALL_STACK: [u8; 4096 * 4] = [0; 4096 * 4];

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
        CPU_STACK.kernel =
            SYSCALL_STACK.as_ptr() as u64 + SYSCALL_STACK.len() as u64;
        KernelGsBase::write(VirtAddr::new(cpu_local));
    }
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_entry() {
    naked_asm!(
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
pub const SYS_EXIT: u64 = 0;
pub const SYS_WRITE: u64 = 1;

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
            0
        },
        SYS_WRITE => {
            let fd = arg1 as usize;
            let buf = arg2 as *const u8;
            let len = arg3 as usize;

            if fd == 1 {
                for i in 0..len {
                    let c = unsafe { *buf.add(i) };
                    print!("{}", c as char);
                }

                return len as u64;
            }
            unreachable!(); // TODO: support other fds
        },
        _ => {
            error!("unknown syscall: {}", num);
            u64::MAX
        },
    }
}
