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

use core::sync::atomic::{AtomicBool, Ordering};

use pci_types::ConfigRegionAccess;
use spin::MutexGuard;
use spin::mutex::Mutex;
use x86_64::VirtAddr;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::PageTableFlags;

use crate::devices::pci::io::PciIO;
use crate::devices::pci::parser::PciBus;
use crate::memory::vmm::AddressSpace;

#[repr(C, packed)]
struct BDL_Entry {
    addr: u32,
    length: u16,
    flags: u16,
}

const AC97_BUFFERS: usize = 32;
const AC97_BUFFER_SIZE: usize = 4096;

const AC97_BDL_VIRT_BASE: u64 = 0xFFFF_FF00_0100_0000;
const AC97_BUFFER_VIRT_BASE: u64 = 0xFFFF_FF00_0101_0000;
const AC97_BUFFER_STRIDE: u64 = 0x1000;

static AC97_INITIALIZED: AtomicBool = AtomicBool::new(false);

struct AudioBuffer {
    virt: VirtAddr,
    phys: u32,
    has_played: bool,
    data_written: usize,
}

struct Ac97 {
    nam: u16,
    nabm: u16,
    entry: usize,
    buffers: [AudioBuffer; AC97_BUFFERS],
    bdl_virt: VirtAddr,
    temp_buffer: [u8; AC97_BUFFER_SIZE],
    temp_count: usize,
    lock: Mutex<()>,
    volume: usize,
}

impl Ac97 {
    fn nam_write(&self, reg: u16, val: u16) {
        let mut port = Port::<u16>::new(self.nam + reg);
        unsafe { port.write(val) }
    }

    fn nabm_write(&self, reg: u16, val: u8) {
        let mut port = Port::<u8>::new(self.nabm + reg);
        unsafe { port.write(val) }
    }

    fn nabm_read(&self, reg: u16) -> u8 {
        let mut port = Port::<u8>::new(self.nabm + reg);
        unsafe { port.read() }
    }

    fn set_volume(&mut self, vol: usize) {
        assert!(vol <= 100);
        self.volume = vol;
        let s = if vol == 0 { 31 } else { (31 * vol) / 100 };
        let chan = 31 - s;
        let combined = (chan as u16) | ((chan as u16) << 8);
        self.nam_write(0x18, combined);
    }

    fn flush(&self) {
        if !AC97_INITIALIZED.load(Ordering::SeqCst) {
            return;
        }

        let mut s = self.nabm_read(0x10 + 0xB);
        if s & 1 != 0 {
            return;
        }
        s |= 1;
        self.nabm_write(0x10 + 0xB, s);
    }

    fn can_write(&self) -> bool {
        if !AC97_INITIALIZED.load(Ordering::SeqCst) {
            return false;
        }
        let read_ptr = self.nabm_read(0x10 + 0x4);
        let buffer_left = if self.entry >= read_ptr as usize {
            let mut left = AC97_BUFFERS - self.entry;
            if read_ptr == 0 && left > 0 {
                left -= 1;
            }
            left
        } else {
            (read_ptr as usize) - self.entry - 1
        };
        buffer_left > 0
    }

    fn setup_buffers(&mut self) {
        let address_space = AddressSpace::current();

        unsafe {
            let bdl_entries = self.bdl_virt.as_mut_ptr::<BDL_Entry>();

            for i in 0..AC97_BUFFERS {
                let vaddr = VirtAddr::new(
                    AC97_BUFFER_VIRT_BASE + (i as u64) * AC97_BUFFER_STRIDE,
                );

                assert!(
                    !address_space.is_mapped(vaddr),
                    "AC97 buffer virt collision at {:#x}",
                    vaddr.as_u64()
                );

                let phys = address_space
                    .map_page_alloc(
                        vaddr,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    )
                    .expect("failed to allocate ac97 buffers");

                self.buffers[i] = AudioBuffer {
                    virt: vaddr,
                    phys: phys.as_u64() as u32,
                    has_played: false,
                    data_written: 0,
                };

                let entry = BDL_Entry {
                    addr: phys.as_u64() as u32,
                    length: (AC97_BUFFER_SIZE / 2) as u16,
                    flags: 0,
                };
                bdl_entries.add(i).write(entry);
            }

            bdl_entries.add(AC97_BUFFERS - 1).as_mut().unwrap().flags |= 0x8000;
        }
    }

    fn write_buffer(&mut self, buf: &[u8]) -> usize {
        assert!(buf.len() < AC97_BUFFER_SIZE);

        let _lock = self.lock.lock();
        let read_ptr = self.nabm_read(0x10 + 0x4);
        let buffer_left = if self.entry >= read_ptr as usize {
            let mut left = AC97_BUFFERS - self.entry;
            if read_ptr == 0 && left > 0 {
                left -= 1;
            }
            left
        } else {
            (read_ptr as usize) - self.entry - 1
        };

        if buffer_left == 0 {
            panic!("no buffer left");
        }

        unsafe {
            let bdl_entries = self.bdl_virt.as_mut_ptr::<BDL_Entry>();
            core::ptr::write_bytes(
                self.buffers[self.entry].virt.as_mut_ptr::<u8>(),
                0,
                AC97_BUFFER_SIZE,
            );
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                self.buffers[self.entry].virt.as_mut_ptr::<u8>(),
                buf.len(),
            );

            let sample_len = buf.len().div_ceil(2) as u16;
            bdl_entries.add(self.entry).as_mut().unwrap().length = sample_len;
        }

        let lvi = self.entry;
        self.nabm_write(0x10 + 0x05, lvi as u8);
        self.entry = (self.entry + 1) % AC97_BUFFERS;
        self.flush();
        buf.len()
    }
}

static AC97_DRIVER: Mutex<Option<Ac97>> = Mutex::new(None);

fn get_driver() -> MutexGuard<'static, Option<Ac97>> { AC97_DRIVER.lock() }

pub fn ready() -> bool { AC97_INITIALIZED.load(Ordering::SeqCst) }

pub fn busy() -> bool {
    let guard = get_driver();
    if let Some(driver) = guard.as_ref() { !driver.can_write() } else { false }
}

pub fn write(buf: &[u8]) -> usize {
    let mut guard = get_driver();
    if let Some(driver) = guard.as_mut() { driver.write_buffer(buf) } else { 0 }
}

pub fn set_volume(vol: usize) {
    let mut guard = get_driver();
    if let Some(driver) = guard.as_mut() {
        driver.set_volume(vol)
    }
}

pub fn install(pci: &PciBus) {
    let address_space = AddressSpace::current();

    if let Some(ac97) = pci.find_by_class(0x04, 0x01) {
        let nam = ac97.bars[0].unwrap().unwrap_io() as u16;
        let nabm = ac97.bars[1].unwrap().unwrap_io() as u16;

        log::debug!("AC97 found, NAM={:#x}, NABM={:#x}", nam, nabm);

        let bdl_virt_addr = VirtAddr::new(AC97_BDL_VIRT_BASE);
        assert!(
            !address_space.is_mapped(bdl_virt_addr),
            "AC97 BDL virt collision at {:#x}",
            bdl_virt_addr.as_u64()
        );

        let bdl_phys = address_space
            .map_page_alloc(
                bdl_virt_addr,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
            .expect("failed to allocate ac97 bdl");
        let bdl_virt = bdl_virt_addr;

        let mut driver = Ac97 {
            nam,
            nabm,
            entry: 0,
            buffers: unsafe { core::mem::zeroed() },
            bdl_virt,
            temp_buffer: [0; AC97_BUFFER_SIZE],
            temp_count: 0,
            lock: Mutex::new(()),
            volume: 50,
        };

        unsafe {
            log::debug!("AC97::MASTERING");
            let pci_io = PciIO;
            let mut cmd = pci_io.read(ac97.addr, 0x04);
            cmd |= 1 << 0;
            cmd |= 1 << 2;
            pci_io.write(ac97.addr, 0x04, cmd);

            log::debug!("AC97::INIT");
            driver.nabm_write(0x2C, 1 << 1);

            log::debug!("AC97::RESET");
            driver.nam_write(0x00, 1);

            log::debug!("AC97::CAPABILITIES");
            driver.nam_write(0x2C, 48000);
            driver.nam_write(0x2E, 48000);
            driver.nam_write(0x30, 48000);
            driver.nam_write(0x32, 48000);

            log::debug!("AC97::VOLUME");
            driver.set_volume(50);

            log::debug!("AC97::PLAY_RESET");
            driver.nam_write(0x02, 0);
            driver.nam_write(0x04, 0);

            log::debug!("AC97::RESET_BIT");
            driver.nabm_write(
                0x10 + 0xB,
                driver.nabm_read(0x10 + 0xB) | (1 << 1),
            );

            log::debug!("AC97::WAITING");
            let mut control = Port::<u8>::new(nabm + 0x10 + 0xB);
            while control.read() & (1 << 0) != 0 {
                core::hint::spin_loop();
            }

            log::debug!("AC97::BUFFER");
            {
                driver.setup_buffers();
            }

            log::debug!("AC97::BDL_ADDR");
            {
                let mut port = Port::<u32>::new(driver.nabm + 0x10);
                port.write(bdl_phys.as_u64() as u32);
            }

            log::debug!("AC97::START");
            driver.nabm_write(
                0x10 + 0xB,
                driver.nabm_read(0x10 + 0xB) | (1 << 0),
            );

            log::info!("AC97 initialized successfully.");

            AC97_INITIALIZED.store(true, Ordering::SeqCst);
            *AC97_DRIVER.lock() = Some(driver);
        }
    } else {
        log::error!("AC97 device not found");
    }
}
