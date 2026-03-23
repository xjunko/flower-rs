use spin::Lazy;
use x86_64::VirtAddr;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{CS, DS, ES, SS, Segment};
use x86_64::structures::gdt::{
    Descriptor, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::structures::tss::TaskStateSegment;

pub struct GDTSegments {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + (STACK_SIZE as u64)
    };

    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + (STACK_SIZE as u64)
    };

    // NOTE: we dont need this anymore because
    //       every user process has it's own stack now.
    // tss.privilege_stack_table[0] = {
    //     const STACK_SIZE: usize = 4096 * 5;
    //     static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

    //     let stack_start = VirtAddr::from_ptr(&raw const STACK);
    //     stack_start + (STACK_SIZE as u64)
    // };

    tss
});

static GDT: Lazy<(GlobalDescriptorTable, GDTSegments)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());

    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());

    let tss = gdt.append(Descriptor::tss_segment(&TSS));

    (gdt, GDTSegments { kernel_code, kernel_data, user_data, user_code, tss })
});

pub fn install() {
    GDT.0.load();

    unsafe {
        CS::set_reg(GDT.1.kernel_code);
        DS::set_reg(GDT.1.kernel_data);
        ES::set_reg(GDT.1.kernel_data);
        SS::set_reg(GDT.1.kernel_data);
        log::info!("GDT loaded.");

        load_tss(GDT.1.tss);
        log::info!("TSS loaded.")
    }
}

pub fn segments() -> &'static GDTSegments { &GDT.1 }

/// sets the kernel stack pointer in the TSS to the given value.
pub fn set_kernel_stack(stack_top: VirtAddr) {
    unsafe {
        (*TSS.as_mut_ptr()).privilege_stack_table[0] = stack_top;
    }
}
