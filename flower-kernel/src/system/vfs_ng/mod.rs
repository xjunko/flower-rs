use alloc::{sync::Arc, vec::Vec};

use crate::system::vfs_ng::inode::INode;

mod inode;
mod file;

#[derive(Default)]
pub enum EntryState {
    Present(Arc<INode>),
    NotPresent,
    #[default]
    NotCached
}

pub struct Entry {
    pub name: Vec<u8>,
    pub inode: EntryState
}
