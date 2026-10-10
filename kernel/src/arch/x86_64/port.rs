use x86_64::instructions::port::Port;
use x86_64::structures::port::{PortRead, PortWrite};

pub fn read<T>(addr: u16) -> T
where T: PortRead {
    let mut port = Port::<T>::new(addr);
    unsafe { port.read() }
}
pub fn write<T>(addr: u16, value: T)
where T: PortWrite {
    let mut port = Port::<T>::new(addr);
    unsafe { port.write(value) }
}
