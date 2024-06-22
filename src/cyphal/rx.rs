use super::CyphalMiddleware;
use super::defines::*;

const CAN_FRAME_XID_AND_HEADER_LENGTH: usize = 8;

impl <const MTU: usize> CyphalMiddleware<MTU> {

    pub fn try_read(&self, data: &[u8]) -> Result<Vec<CyphalRxPacket<MTU>>, Box<dyn std::error::Error>> {

        let data_len: usize = data.len();
        if data_len % (MTU + CAN_FRAME_XID_AND_HEADER_LENGTH) != 0 {
            return Err(String::from("INVALID DATA RECEIVED: UNACCEPTABLE FRAME LENGTH").into());
        };

        let vecs: Vec<Vec<u8>> = data
            .chunks(MTU + CAN_FRAME_XID_AND_HEADER_LENGTH)
            .map(|x| Vec::from(x))
            .collect();

        let mut ret: Vec<CyphalRxPacket<MTU>> = vec![];
        
        for v in vecs {

            // CAN ID Field
            let xid: u32 = u32::from_le_bytes([v[0], v[1], v[2], v[3]]) & CAN_EXT_ID_MASK;

            //CAN Header Field
            let header: u32 = u32::from_le_bytes([v[4], v[5], v[6], v[7]]);
            let dlc: u8 = ((header >> 16) & 0x0f) as u8;
            // let mm: u8 = (header >> 24) as u8;
            // let fdf: u8 = ((header >> 21) & 0x01) as u8;
            // let brs: u8 = ((header >> 20) & 0x01) as u8;

            let dlen: usize = CAN_DLC_TO_DLEN[dlc as usize] as usize;
            let tail: u8 = v[CAN_FRAME_XID_AND_HEADER_LENGTH + dlen - 1];
            let transfer_id: u8 = tail & CYPHAL_TRANSFER_ID_MAX;
            let start_of_transfer: bool = (tail & TAIL_START_OF_TRANSFER) != 0;
            let end_of_transfer: bool = (tail & TAIL_END_OF_TRANSFER) != 0;
            let toggle: bool = (tail & TAIL_TOGGLE) != 0;
            let frame_type = match (start_of_transfer, end_of_transfer) {
                (true, true) => CyphalRxPacketType::SignleFrame,
                (true, false) => CyphalRxPacketType::MultiFrameStart,
                (false, true) => CyphalRxPacketType::MultiFrameEnd,
                (false, false) => CyphalRxPacketType::MultiFrameInProcess
            };

            //CAN Data Field
            let dlen: usize = dlen - TAIL_SIZE_BYTES as usize;
            let mut payload: [u8; MTU] = [0; MTU];
            for i in 0..dlen {
                payload[i] = v[i + CAN_FRAME_XID_AND_HEADER_LENGTH];
            }
            let payload_size: usize = dlen;
            
            //Construct Cyphal Rx Packet (with status and property) 
            let transfer_kind: CyphalTransferKind = match ((xid & FLAG_SERVICE_NOT_MESSAGE) >> 25) == 1 {
                true => {
                    match ((xid & FLAG_REQUEST_NOT_RESPONSE) >> 24) == 1 {
                        true => CyphalTransferKind::Request,
                        false => CyphalTransferKind::Response,
                    }
                }
                false => CyphalTransferKind::Message,
            };

            let (priority, port_id, source_node_id, destination_node_id) = match transfer_kind {
                CyphalTransferKind::Message => {
                    let priority: CyphalPriority = CyphalPriority::from(((xid >> OFFSET_PRIORITY) & CYPHAL_PRIORITY_MAX as u32) as u8);
                    let port_id: CyphalPortID = ((xid >> OFFSET_SUBJECT_ID as u32) & CYPHAL_SUBJECT_ID_MAX as u32) as u16;
                    let source_node_id: CyphalNodeID = if (xid & FLAG_ANONYMOUS_MESSAGE) != 0 { CYPHAL_NODE_ID_UNSET } else { (xid & CYPHAL_NODE_ID_MAX as u32) as u8 };
                    let destination_node_id: CyphalNodeID = CYPHAL_NODE_ID_UNSET;
                    (priority, port_id, source_node_id, destination_node_id)
                },
                _ => {
                    let priority: CyphalPriority = CyphalPriority::from(((xid >> OFFSET_PRIORITY) & CYPHAL_PRIORITY_MAX as u32) as u8);
                    let port_id: CyphalPortID = ((xid >> OFFSET_SERVICE_ID as u32) & CYPHAL_SERVICE_ID_MAX as u32) as u16; 
                    let source_node_id: CyphalNodeID = (xid & CYPHAL_NODE_ID_MAX as u32) as u8;
                    let destination_node_id: CyphalNodeID = ((xid >> OFFSET_DST_NODE_ID as u32) as u8) & CYPHAL_NODE_ID_MAX;
                    (priority, port_id, source_node_id, destination_node_id)
                },
            };

            ret.push(CyphalRxPacket {
                xid,
                payload,
                payload_size,
                status: CyphalRxPacketStatus {
                    frame_type,
                    toggle,
                },
                props: CyphalRxProps{
                    priority,
                    transfer_id,
                    transfer_kind,
                    port_id,
                    source_node_id,
                    destination_node_id
                },
            });
        }

        Ok(ret)
    }


}
