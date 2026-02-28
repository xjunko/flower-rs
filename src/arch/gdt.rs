use spin::Lazy;
use x86_64::{
    VirtAddr,
    instructions::tables::load_tss,
    registers::segmentation::{CS, DS, SS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};

pub struct GDTSegments {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + (STACK_SIZE as u64)
    };

    tss.privilege_stack_table[0] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + (STACK_SIZE as u64)
    };

    tss
});

static GDT: Lazy<(GlobalDescriptorTable, GDTSegments)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());

    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());

    let tss = gdt.append(Descriptor::tss_segment(&TSS));

    (
        gdt,
        GDTSegments {
            kernel_code,
            kernel_data,
            user_data,
            user_code,
            tss,
        },
    )
});

pub fn install() {
    GDT.0.load();

    unsafe {
        CS::set_reg(GDT.1.kernel_code);
        DS::set_reg(GDT.1.kernel_data);
        SS::set_reg(SegmentSelector(0));
        load_tss(GDT.1.tss);
    }
}

pub fn segments() -> &'static GDTSegments {
    &GDT.1
}

pub fn set_kernel_stack(stack_top: VirtAddr) {
    unsafe {
        // NOTE: is this the right way of doing it?
        (*TSS.as_mut_ptr()).privilege_stack_table[0] = stack_top;
    }
}
