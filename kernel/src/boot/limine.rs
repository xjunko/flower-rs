use limine::request::{
    FramebufferRequest, HhdmRequest, MemmapRequest, ModulesRequest, MpRequest,
    RsdpRequest, StackSizeRequest,
};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

use crate::arch::layout::KERNEL_STACK_SIZE;

#[used]
#[unsafe(link_section = ".limine_requests_start")]
pub static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static SMP_REQUEST: MpRequest = MpRequest::new(0);

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static MODULE_REQUESTS: ModulesRequest = ModulesRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static STACK_SIZE_REQUEST: StackSizeRequest =
    StackSizeRequest::new(KERNEL_STACK_SIZE as u64);

#[used]
#[unsafe(link_section = ".limine_requests_end")]
pub static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
