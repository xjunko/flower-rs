use flower_abi::system::{UTSNAME_LEN, UtsName};
use spin::mutex::SpinMutex;

const SYSNAME: &[u8] = b"flower-rs";
const NODENAME: &[u8] = b"localhost";
const RELEASE: &[u8] = env!("CARGO_PKG_VERSION").as_bytes();
const VERSION: &[u8] = b"default";
const DOMAINNAME: &[u8] = b"(none)";

#[cfg(target_arch = "x86_64")]
const MACHINE: &[u8] = b"x86_64";

pub static UTSNAME: SpinMutex<UtsName> = SpinMutex::new({
    let mut name = UtsName {
        sysname: [0; UTSNAME_LEN],
        nodename: [0; UTSNAME_LEN],
        release: [0; UTSNAME_LEN],
        version: [0; UTSNAME_LEN],
        machine: [0; UTSNAME_LEN],
        domainname: [0; UTSNAME_LEN],
    };

    name.sysname[0..SYSNAME.len()].copy_from_slice(SYSNAME);
    name.nodename[0..NODENAME.len()].copy_from_slice(NODENAME);
    name.release[0..RELEASE.len()].copy_from_slice(RELEASE);
    name.version[0..VERSION.len()].copy_from_slice(VERSION);
    name.domainname[0..DOMAINNAME.len()].copy_from_slice(DOMAINNAME);
    name.machine[0..MACHINE.len()].copy_from_slice(MACHINE);

    name
});
