/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

use limine::request::{
    FramebufferRequest, HhdmRequest, MemmapRequest, ModulesRequest, MpRequest,
    RsdpRequest, StackSizeRequest,
};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

use crate::arch::x86_64::layout::KERNEL_STACK_SIZE;

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
