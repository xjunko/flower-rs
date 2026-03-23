use core::ffi::CStr;
use core::sync::atomic::{AtomicUsize, Ordering};

pub const AT_NULL: usize = 0;
pub const AT_PHDR: usize = 3;
pub const AT_PHENT: usize = 4;
pub const AT_PHNUM: usize = 5;
pub const AT_PAGESZ: usize = 6;
pub const AT_ENTRY: usize = 9;

static AUXV_BASE: AtomicUsize = AtomicUsize::new(0);
static ARGC: AtomicUsize = AtomicUsize::new(0);
static ARGV_BASE: AtomicUsize = AtomicUsize::new(0);

const MAX_SCAN_WORDS: usize = 512;
const MAX_ARGC: usize = 128;
const MAX_ENVC: usize = 256;
const MAX_AUXV_PAIRS: usize = 128;
const MIN_VALID_USER_PTR: usize = 0x1000;

struct ParsedStackLayout {
    auxv_base: *const usize,
    argc: usize,
    argv_base: *const usize,
}

unsafe fn parse_auxv_base_from_stack(
    stack_base: *const usize,
    scan_limit: usize,
) -> Option<ParsedStackLayout> {
    let can_read_word = |ptr: *const usize| {
        let addr = ptr as usize;
        addr.checked_add(core::mem::size_of::<usize>())
            .is_some_and(|end| end <= scan_limit)
    };

    if !can_read_word(stack_base) {
        return None;
    }

    let argc = unsafe { *stack_base };
    if argc > MAX_ARGC {
        return None;
    }

    let argv_base = unsafe { stack_base.add(1) };
    for idx in 0..argc {
        let argv_slot = unsafe { argv_base.add(idx) };
        if !can_read_word(argv_slot) {
            return None;
        }

        let arg_ptr = unsafe { *argv_slot };
        if arg_ptr < MIN_VALID_USER_PTR {
            return None;
        }
    }

    let mut ptr = unsafe { stack_base.add(1 + argc) };
    if !can_read_word(ptr) {
        return None;
    }
    if unsafe { *ptr } != 0 {
        return None;
    }

    ptr = unsafe { ptr.add(1) };

    let mut env_count = 0;
    loop {
        if !can_read_word(ptr) {
            return None;
        }

        if unsafe { *ptr } == 0 {
            break;
        }

        env_count += 1;
        if env_count > MAX_ENVC {
            return None;
        }

        ptr = unsafe { ptr.add(1) };
    }

    ptr = unsafe { ptr.add(1) };
    if !can_read_word(ptr) {
        return None;
    }

    let mut aux_pairs = 0;
    let mut has_entry = false;
    loop {
        if !can_read_word(ptr) || !can_read_word(unsafe { ptr.add(1) }) {
            return None;
        }

        let item_key = unsafe { *ptr };
        let _item_value = unsafe { *ptr.add(1) };

        if item_key == AT_NULL {
            break;
        }

        if item_key == AT_ENTRY {
            has_entry = true;
        }

        aux_pairs += 1;
        if aux_pairs > MAX_AUXV_PAIRS {
            return None;
        }

        ptr = unsafe { ptr.add(2) };
    }

    if !has_entry {
        return None;
    }

    Some(ParsedStackLayout {
        auxv_base: unsafe { stack_base.add(1 + argc + 1 + env_count + 1) },
        argc,
        argv_base,
    })
}

unsafe fn init_from_rsp(rsp: usize) {
    let rsp_ptr = rsp as *const usize;
    let scan_limit = (rsp + 0x1000) & !0xFFF;

    if let Some(layout) =
        unsafe { parse_auxv_base_from_stack(rsp_ptr, scan_limit) }
    {
        AUXV_BASE.store(layout.auxv_base as usize, Ordering::Relaxed);
        ARGC.store(layout.argc, Ordering::Relaxed);
        ARGV_BASE.store(layout.argv_base as usize, Ordering::Relaxed);
        return;
    }

    for offset in 1..=MAX_SCAN_WORDS {
        let candidate = unsafe { rsp_ptr.add(offset) };
        if let Some(layout) =
            unsafe { parse_auxv_base_from_stack(candidate, scan_limit) }
        {
            AUXV_BASE.store(layout.auxv_base as usize, Ordering::Relaxed);
            ARGC.store(layout.argc, Ordering::Relaxed);
            ARGV_BASE.store(layout.argv_base as usize, Ordering::Relaxed);
            return;
        }
    }

    AUXV_BASE.store(0, Ordering::Relaxed);
    ARGC.store(0, Ordering::Relaxed);
    ARGV_BASE.store(0, Ordering::Relaxed);
}

/// # Safety
///
/// this function must be called exactly once, at the start of any program.
///
/// initializes the auxv by trying to scan the stack for auxv base.
pub unsafe fn init_current() {
    let rsp: usize;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
    }
    unsafe { init_from_rsp(rsp) };
}

pub fn getauxval(key: usize) -> Option<usize> {
    let mut ptr = AUXV_BASE.load(Ordering::Relaxed) as *const usize;
    if ptr.is_null() {
        return None;
    }

    loop {
        let item_key = unsafe { *ptr };
        let item_value = unsafe { *ptr.add(1) };

        if item_key == AT_NULL {
            return None;
        }

        if item_key == key {
            return Some(item_value);
        }

        ptr = unsafe { ptr.add(2) };
    }
}

pub fn argc() -> usize { ARGC.load(Ordering::Relaxed) }

pub fn argv(index: usize) -> Option<&'static str> {
    if index >= argc() {
        return None;
    }

    let argv_base = ARGV_BASE.load(Ordering::Relaxed) as *const usize;
    if argv_base.is_null() {
        return None;
    }

    let ptr = unsafe { *argv_base.add(index) } as *const i8;
    if ptr.is_null() {
        return None;
    }

    if (ptr as usize) < MIN_VALID_USER_PTR {
        return None;
    }

    unsafe { CStr::from_ptr(ptr).to_str().ok() }
}
