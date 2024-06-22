use super::*;
use crate::cyphal::crc_add;

pub struct CyphalInstance<const MTU: usize> {
    pub(crate) mtu_bytes: usize,
    pub(crate) node_id: CyphalNodeID,
}

impl <const MTU: usize> CyphalInstance<MTU> {
    pub fn new(node_id: CyphalNodeID) -> Self {
        Self {
            mtu_bytes: MTU,
            node_id,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CyphalTxProps {
    pub priority: CyphalPriority,
    pub transfer_kind: CyphalTransferKind,
    pub transfer_id: CyphalTransferID,
    pub port_id: CyphalPortID,
    pub remote_node_id: CyphalNodeID,
}

#[derive(Debug)]
pub struct CyphalTxPacketFrame {
    pub payload: Vec<u8>,
    pub payload_size: usize,
    pub props: CyphalTxProps
}

#[derive(Debug, Clone)]
pub struct CyphalTxPacket<const MTU: usize> {
    pub xid: u32,
    pub payload: [u8; MTU],
    pub payload_size: usize,
}


#[derive(Debug, Copy, Clone)]
pub struct CyphalRxPacketStatus {
    pub frame_type: CyphalRxPacketType,
    pub toggle: bool
}

#[derive(Debug, Copy, Clone)]
pub struct CyphalRxProps {
    pub priority: CyphalPriority,
    pub transfer_kind: CyphalTransferKind,
    pub transfer_id: CyphalTransferID,
    pub port_id: CyphalPortID,
    pub source_node_id: CyphalNodeID,
    pub destination_node_id: CyphalNodeID,
}

#[derive(Debug, Clone)]
pub struct CyphalRxPacket<const MTU: usize> {
    pub xid: u32,
    pub payload: [u8; MTU],
    pub payload_size: usize,
    pub status: CyphalRxPacketStatus,
    pub props: CyphalRxProps
}

#[derive(Debug, Clone)]
pub struct CyphalRxFrame {
    pub xid: u32,
    pub payload: Vec<u8>,
    pub payload_size: usize,
    pub props: CyphalRxProps
}

impl CyphalRxFrame {
    pub fn calculate_crc(&self) -> Result<[u8; CRC_SIZE_BYTES as usize], Box<dyn std::error::Error>> {
        let crc = crc_add(CRC_INITIAL, self.payload_size, &self.payload)?;
        Ok([(crc >> BITS_PER_BYTE) as u8, (crc & BYTE_MAX) as u8])
    }
}

#[derive(Debug)]
pub struct CyphalRxData <T> {
  pub data: T,
  pub props: CyphalRxProps
} 

