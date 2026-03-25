use alloc::vec::Vec;

use crate::auxv;

pub struct Args<'a> {
    args: Vec<&'a str>,
    index: usize,
}

impl<'a> Args<'a> {
    pub fn new() -> Self {
        let mut args = Vec::new();

        for i in 0..auxv::argc() {
            if let Some(arg) = auxv::argv(i) {
                args.push(arg);
            }
        }

        Args { args, index: 0 }
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
