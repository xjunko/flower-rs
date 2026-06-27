use alloc::vec::Vec;

use crate::platform;

pub struct Args<'a> {
    args: Vec<&'a str>,
    index: usize,
}

impl<'a> Args<'a> {
    #[allow(static_mut_refs)]
    pub fn new() -> Self {
        Args { args: unsafe { platform::argv.clone() }, index: 0 }
    }
}

impl<'a> Default for Args<'a> {
    fn default() -> Self { Self::new() }
}

impl<'a> Iterator for Args<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.args.len() {
            return None;
        }

        let item = self.args[self.index];
        self.index += 1;
        Some(item)
    }
}

pub fn args<'a>() -> Args<'a> { Args::new() }

#[allow(static_mut_refs)]
pub fn argv(i: usize) -> Option<&'static str> {
    let args = unsafe { &platform::argv };
    if i < args.len() { Some(args[i]) } else { None }
}

#[allow(static_mut_refs)]
pub fn argc() -> usize {
    let args = unsafe { &platform::argv };
    args.len()
}
