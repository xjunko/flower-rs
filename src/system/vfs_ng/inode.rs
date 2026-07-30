use alloc::sync::Arc;



pub struct INode {
    pub node_ops:
}


pub enum NodeOps {
    Regular(Arc<dyn RegularOps>)
}
