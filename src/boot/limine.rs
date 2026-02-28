use limine::{
    BaseRevision,
    request::{
        FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest, RequestsEndMarker,
        RequestsStartMarker, RsdpRequest,
    },
};

#[used]
#[unsafe(link_section = ".limine_requests_start")]
pub static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static MODULE_REQUESTS: ModuleRequest = ModuleRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
pub static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
