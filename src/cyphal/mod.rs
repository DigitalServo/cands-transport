pub mod defines;
pub use defines::*;

mod tx;
mod rx;
mod crc;

pub use crc::*;

pub struct CyphalMiddleware <const MTU: usize> {
    can_instance: CyphalInstance<MTU>,
    pub transfer_id: u8,
}

impl <const MTU: usize> CyphalMiddleware<MTU> {
    pub fn new(node_id: CyphalNodeID) -> Self {
        Self {
            can_instance: CyphalInstance::new(node_id),
            transfer_id: 0
        }
    }

    pub fn set_node_id(mut self, node_id: u8) -> Self {
        self.can_instance.node_id = node_id;
        self
    }
}
