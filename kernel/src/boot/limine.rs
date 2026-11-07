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

use limine::BaseRevision;
use limine::memory_map::EntryType;
use limine::request::{
    FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest,
    PagingModeRequest, RequestsEndMarker, RequestsStartMarker, RsdpRequest,
    StackSizeRequest,
};
use spin::mutex::SpinMutex;

use crate::arch::addr::{KPhysAddr, KVirtAddr};
use crate::arch::layout::KERNEL_STACK_SIZE;
use crate::boot::{KBootFile, KBootInfo, KMemoryMap, KMemoryMapType};

#[used]
#[unsafe(link_section = ".limine_requests_start")]
pub static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(3);

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".boot")]
pub static PAGING_REQUEST: PagingModeRequest = PagingModeRequest::new();

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
#[unsafe(link_section = ".limine_requests")]
pub static STACK_SIZE_REQUEST: StackSizeRequest =
    StackSizeRequest::new().with_size(KERNEL_STACK_SIZE as u64);

#[used]
#[unsafe(link_section = ".limine_requests_end")]
pub static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

// kernel side
static mut MEMMAP_BUF: [KMemoryMap; 128] = [KMemoryMap::empty(); _];
static mut FILE_BUFFER: [KBootFile; 32] = [KBootFile::new(); _];

pub(crate) fn install() {
    let mut info = KBootInfo::new();

    // memory
    {
        let entries =
            MEMORY_MAP_REQUEST.get_response().expect("no mmap").entries();
        let mut total_entries = 0;
        entries.iter().enumerate().for_each(|(i, entry)| unsafe {
            MEMMAP_BUF[i] = KMemoryMap {
                length: entry.length as usize,
                address: KPhysAddr::new(entry.base),
                usage: match entry.entry_type {
                    EntryType::USABLE => KMemoryMapType::Usable,
                    EntryType::BOOTLOADER_RECLAIMABLE
                    | EntryType::EXECUTABLE_AND_MODULES => {
                        KMemoryMapType::Reclaimable
                    },
                    _ => KMemoryMapType::Reserved,
                },
            };

            total_entries += 1
        });

        info.highest_phys = Some({
            let last = entries.iter().last().unwrap();
            KPhysAddr::new(last.base + last.length)
        });
        log::debug!(
            "boot: highest_phys: {:#x}",
            info.highest_phys.expect("no highest phys")
        );

        info.hhdm = Some(KVirtAddr::new(
            HHDM_REQUEST.get_response().expect("no hhdm").offset(),
        ));
        log::debug!("boot: hddm at {:#x}", info.hhdm.unwrap());

        unsafe {
            info.memory = SpinMutex::new(&MEMMAP_BUF[0..total_entries]);
        }
        log::debug!("boot: found {} memory entries", total_entries);
    }

    // rsdp
    info.rsdp_addr = RSDP_REQUEST
        .get_response()
        .map(|x| KPhysAddr::new((x.address()) as u64));
    log::debug!("boot: rsdp at {:#x}", info.rsdp_addr.expect("no rsdp"));

    // modules
    if let Some(response) = MODULE_REQUESTS.get_response() {
        for (i, entry) in response.modules().iter().enumerate() {
            unsafe {
                FILE_BUFFER[i] = KBootFile {
                    data: KPhysAddr::new(
                        (entry.addr() as u64)
                            .wrapping_sub(info.hhdm.unwrap().as_u64()),
                    ),
                    length: entry.size() as usize,
                    path: entry
                        .path()
                        .to_str()
                        .expect("failed to parse module path"),
                }
            }
        }

        unsafe {
            info.files = &FILE_BUFFER[0..response.modules().len()];
        }
        log::debug!("boot: found {} modules", info.files.len());
    }

    info.register();
}
