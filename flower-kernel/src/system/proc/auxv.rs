use alloc::vec::Vec;

use x86_64::VirtAddr;

use crate::system::elf;
use crate::system::mem::vmm::AddressSpace;
use crate::system::proc::PAGE_SIZE;

const AT_NULL: u64 = 0;
const AT_PHDR: u64 = 3;
const AT_PHENT: u64 = 4;
const AT_PHNUM: u64 = 5;
const AT_PAGESZ: u64 = 6;
const AT_ENTRY: u64 = 9;

fn push_user_u64(
    address_space: &AddressSpace,
    stack_low: u64,
    stack_ptr: &mut u64,
    value: u64,
) -> Result<u64, &'static str> {
    *stack_ptr = stack_ptr.checked_sub(8).ok_or("user stack overflow")?;
    if *stack_ptr < stack_low {
        return Err("user stack overflow");
    }
    address_space.write(VirtAddr::new(*stack_ptr), &value.to_ne_bytes())?;
    Ok(*stack_ptr)
}

fn push_user_bytes(
    address_space: &AddressSpace,
    stack_low: u64,
    stack_ptr: &mut u64,
    bytes: &[u8],
) -> Result<u64, &'static str> {
    let len = bytes.len() as u64;
    *stack_ptr = stack_ptr.checked_sub(len).ok_or("user stack overflow")?;
    if *stack_ptr < stack_low {
        return Err("user stack overflow");
    }

    address_space.write(VirtAddr::new(*stack_ptr), bytes)?;
    Ok(*stack_ptr)
}

/// builds the initial user stack with the given program name and ELF information.
pub fn build_initial_user_stack(
    argv: &[&str],
    address_space: &AddressSpace,
    stack_low: u64,
    mut stack_top: u64,
    loaded: &elf::ELF64,
) -> Result<u64, &'static str> {
    let mut argv_ptrs = Vec::with_capacity(argv.len());
    for arg in argv.iter().rev() {
        let mut arg_bytes = Vec::from(arg.as_bytes());
        arg_bytes.push(0);
        let arg_ptr = push_user_bytes(
            address_space,
            stack_low,
            &mut stack_top,
            &arg_bytes,
        )?;
        argv_ptrs.push(arg_ptr);
    }
    argv_ptrs.reverse();

    stack_top &= !0xF;
    if stack_top < stack_low {
        return Err("user stack overflow");
    }

    let mut aux_words = Vec::new();
    if loaded.phdr != 0 {
        aux_words.push(AT_PHDR);
        aux_words.push(loaded.phdr);
    }
    aux_words.push(AT_PHENT);
    aux_words.push(loaded.phent);
    aux_words.push(AT_PHNUM);
    aux_words.push(loaded.phnum);
    aux_words.push(AT_PAGESZ);
    aux_words.push(PAGE_SIZE);
    aux_words.push(AT_ENTRY);
    aux_words.push(loaded.entry);
    aux_words.push(AT_NULL);
    aux_words.push(0);

    for word in aux_words.iter().rev() {
        push_user_u64(address_space, stack_low, &mut stack_top, *word)?;
    }

    push_user_u64(address_space, stack_low, &mut stack_top, 0)?;
    push_user_u64(address_space, stack_low, &mut stack_top, 0)?;
    for arg_ptr in argv_ptrs.iter().rev() {
        push_user_u64(address_space, stack_low, &mut stack_top, *arg_ptr)?;
    }
    push_user_u64(
        address_space,
        stack_low,
        &mut stack_top,
        argv_ptrs.len() as u64,
    )?;

    Ok(stack_top)
}
