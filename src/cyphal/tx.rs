use std::borrow::Borrow;

use super::CyphalMiddleware;
use super::defines::*;
use super::crc::{crc_add, crc_add_byte};

// Public functions
impl <const MTU: usize> CyphalMiddleware<MTU> {
    pub fn create_heartbeat_tx_data(&mut self) -> Result<Vec<CyphalTxPacket<MTU>>, Box<dyn std::error::Error>> {
        let hb_sbject_id: u16 = 0x1d55; // HeardBeat subject-ID.
        let transfer_data: CyphalTxPacketFrame = CyphalTxPacketFrame {
            props: CyphalTxProps {
                priority: CyphalPriority::Nominal,
                transfer_kind: CyphalTransferKind::Message,
                transfer_id: self.transfer_id,
                port_id: hb_sbject_id,
                remote_node_id: CYPHAL_NODE_ID_UNSET,
            },
            payload_size: 7,
            payload: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa1],
        };
        self.transfer_id = if self.transfer_id == 255 { 0 } else { self.transfer_id + 1 };
        self.create_packet(transfer_data)
    }

    pub fn create_message_data(
        &mut self,
        subject_id: u16,
        data: &[u8],
        data_size: usize
    ) -> Result<Vec<CyphalTxPacket<MTU>>, Box<dyn std::error::Error>> {
        let transfer_data: CyphalTxPacketFrame = CyphalTxPacketFrame {
            props: CyphalTxProps {
                priority: CyphalPriority::Nominal,
                transfer_kind: CyphalTransferKind::Message,
                transfer_id: self.transfer_id,
                port_id: subject_id,
                remote_node_id: CYPHAL_NODE_ID_UNSET,
            },
            payload_size: data_size,
            payload: Vec::from(data),
        };
        self.transfer_id = if self.transfer_id == 255 { 0 } else { self.transfer_id + 1 };
        self.create_packet(transfer_data)
    }

    pub fn create_request_data(
        &mut self,
        remote_node_id: u8,
        port_id: u16,
        data: &[u8],
        data_size: usize
    ) -> Result<Vec<CyphalTxPacket<MTU>>, Box<dyn std::error::Error>> {
        let transfer_data: CyphalTxPacketFrame = CyphalTxPacketFrame {
            props: CyphalTxProps {
                priority: CyphalPriority::Nominal,
                transfer_kind: CyphalTransferKind::Request,
                transfer_id: self.transfer_id,
                port_id,
                remote_node_id,
            },
            payload_size: data_size,
            payload: Vec::from(data),
        };
        self.transfer_id = if self.transfer_id == 255 { 0 } else { self.transfer_id + 1 };
        self.create_packet(transfer_data)
    }

    pub fn create_response_data(
        &mut self,
        remote_node_id: u8,
        port_id: u16,
        data: &[u8],
        data_size: usize
    ) -> Result<Vec<CyphalTxPacket<MTU>>, Box<dyn std::error::Error>> {
        let transfer_data: CyphalTxPacketFrame = CyphalTxPacketFrame {
            props: CyphalTxProps {
                priority: CyphalPriority::Nominal,
                transfer_kind: CyphalTransferKind::Response,
                transfer_id: self.transfer_id,
                port_id,
                remote_node_id,
            },
            payload_size: data_size,
            payload: Vec::from(data),
        };
        self.transfer_id = if self.transfer_id == 255 { 0 } else { self.transfer_id + 1 };
        self.create_packet(transfer_data)
    }
}

// Private functions
impl <const MTU: usize> CyphalMiddleware<MTU> {
    fn create_packet<T: Borrow<CyphalTxPacketFrame>>(&self, transfer_data: T) -> Result<Vec<CyphalTxPacket<MTU>>, Box<dyn std::error::Error>> {
        let transfer_data: &CyphalTxPacketFrame = transfer_data.borrow();
        let pl_mtu: u8 = self.tx_get_presentation_layer_mtu();
        let can_id: u32 = match self.tx_make_can_id(transfer_data, self.can_instance.node_id) {
            Ok(x) => x,
            Err(err) => return Err(err)
        };
        if can_id > 0 {
            if transfer_data.payload_size <= pl_mtu as usize {
                match self.handle_single_frame(can_id, transfer_data) {
                    Ok(x) => Ok(vec![x]),
                    Err(err) => Err(err)
                }
            }
            else {
                match self.handle_multi_frame(can_id, transfer_data) {
                    Ok(x) => Ok(x),
                    Err(err) => Err(err)
                }
            }
        }
        else {
            Err("INVALID CAN ID".into())
        }
    }

    fn handle_single_frame(&self, xid: u32, transfer_data: &CyphalTxPacketFrame) -> Result<CyphalTxPacket<MTU>, Box<dyn std::error::Error>> {
        if transfer_data.payload.len() != transfer_data.payload_size {
            return Err("INVALID PAYLOAD LENGTH".into());
        };

        let frame_payload_size: usize = match Self::tx_round_frame_payload_sizeup(transfer_data.payload_size + 1) {
            Ok(x) => x,
            Err(err) => return Err(err)
        };

        if frame_payload_size > self.can_instance.mtu_bytes {
            return Err("INVALID FRAME LENGTH".into());
        };

        let mut payload: [u8; MTU] = [0; MTU];
        for i in 0..transfer_data.payload_size {
            payload[i] = transfer_data.payload[i];
        }

        match Self::tx_make_tail_byte(true, true, true, transfer_data.props.transfer_id) {
            Ok(tail_byte) => payload[frame_payload_size - 1] = tail_byte,
            Err(err) => return Err(err)
        };

        let payload_size: usize = frame_payload_size;
        
        Ok(CyphalTxPacket { xid, payload, payload_size })
    }

    fn handle_multi_frame(&self, xid: u32, transfer_data: &CyphalTxPacketFrame) -> Result<Vec<CyphalTxPacket<MTU>>, Box<dyn std::error::Error>> {

        let pl_mtu: u8 = self.tx_get_presentation_layer_mtu();

        let payload_size_with_crc: usize = transfer_data.payload_size + CRC_SIZE_BYTES as usize;
        let mut offset: usize = 0;
        let mut crc: TransferCRC = crc_add(CRC_INITIAL, transfer_data.payload_size, &transfer_data.payload)?;
        let mut start_of_transfer: bool = true;
        let mut toggle: bool = INITIAL_TOGGLE_STATE;

        let mut packets: Vec<CyphalTxPacket<MTU>> = vec![];

        while offset < payload_size_with_crc {
            let residual_payload_size: usize = payload_size_with_crc - offset;
            let frame_payload_size_with_tail: usize = if residual_payload_size < pl_mtu as usize {
                Self::tx_round_frame_payload_sizeup(residual_payload_size + 1)?
            } else {
                pl_mtu as usize + 1
            };

            let frame_payload_size: usize = frame_payload_size_with_tail - 1;

            let mut payload: [u8; MTU] = [0; MTU];
            
            let mut frame_offset: usize = 0;
            let payload_size_in_frame: usize = match offset < transfer_data.payload_size {
                true => std::cmp::min(transfer_data.payload_size - offset, frame_payload_size),
                false => 0
            };

            for i in 0..payload_size_in_frame {
                payload[i] = transfer_data.payload[i + offset];
            }

            frame_offset += payload_size_in_frame;
            offset += payload_size_in_frame;

            // Handle the last frame of the transfer: it is special because it also contains padding and CRC.
            if offset >= transfer_data.payload_size {
                // Insert padding -- only in the last frame. Don't forget to include padding into the CRC.
                while (frame_offset + CRC_SIZE_BYTES as usize) < frame_payload_size {
                    payload[frame_offset] = PADDING_BYTE_VALUE;
                    frame_offset += 1;
                    crc = crc_add_byte(crc, PADDING_BYTE_VALUE);
                }

                // Insert the CRC.
                if (frame_offset < frame_payload_size) && (offset == transfer_data.payload_size) {
                    payload[frame_offset] = (crc >> BITS_PER_BYTE) as u8;
                    frame_offset += 1;
                    offset += 1;
                }
                if (frame_offset < frame_payload_size) && (offset > transfer_data.payload_size) {
                    payload[frame_offset] = (crc & BYTE_MAX) as u8;
                    frame_offset += 1;
                    offset += 1;
                }
            }

            // Finalize the frame.
            match Self::tx_make_tail_byte(start_of_transfer, offset >= payload_size_with_crc, toggle, transfer_data.props.transfer_id) {
                Ok(tail_byte) => payload[frame_offset] = tail_byte,
                Err(err) => return Err(err)
            };

            start_of_transfer = false;
            toggle = !toggle;
    
            packets.push(CyphalTxPacket {
                xid,
                payload,
                payload_size: frame_payload_size_with_tail
            });
        }

        Ok(packets)
    }

    fn tx_get_presentation_layer_mtu(&self) -> u8 {
        const MAX_INDEX: u8 = 64;
        let mtu: u8 = if self.can_instance.mtu_bytes < (CYPHAL_MTU_CAN_CLASSIC as usize) {
            CYPHAL_MTU_CAN_CLASSIC
        } else if self.can_instance.mtu_bytes < (MAX_INDEX as usize) {
            CAN_DLC_TO_DLEN[CAN_DLEN_TO_DLC[self.can_instance.mtu_bytes as usize] as usize]
        } else {
            CAN_DLC_TO_DLEN[CAN_DLEN_TO_DLC[MAX_INDEX as usize] as usize]
        };
        mtu - 1
    }

    fn tx_make_message_session_specifier(
        subject_id: u16,
        src_node_id: u8
    ) -> Result<u32, Box<dyn std::error::Error>> {
        if src_node_id > CYPHAL_NODE_ID_MAX {
            return Err("INVALID NODE ID".into())
        };
        if subject_id > CYPHAL_SUBJECT_ID_MAX {
            return Err("INVALID SUBJECT ID".into())
        };

        let tmp: u32 = subject_id as u32 | (CYPHAL_SUBJECT_ID_MAX + 1) as u32 | ((CYPHAL_SUBJECT_ID_MAX + 1) * 2) as u32;
        Ok(src_node_id as u32 | (tmp << OFFSET_SUBJECT_ID))
    }

    fn tx_make_service_session_specifier(
        service_id: CyphalPortID,
        request_not_response: bool,
        src_node_id: CyphalNodeID,
        dst_node_id: CyphalNodeID,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        if src_node_id > CYPHAL_NODE_ID_MAX {
            return Err("INVALID SRC NODE ID".into());
        }
        if dst_node_id > CYPHAL_NODE_ID_MAX {
            return Err("INVALID DST NODE ID".into());
        }
        if service_id > CYPHAL_SERVICE_ID_MAX {
            return Err("INVALID SERVICE ID".into());
        }
        let mut ret: u32 = 0;
        ret |= src_node_id as u32;
        ret |= (dst_node_id as u32) << OFFSET_DST_NODE_ID;
        ret |= (service_id as u32) << OFFSET_SERVICE_ID;
        ret |= if request_not_response { FLAG_REQUEST_NOT_RESPONSE } else { 0 };
        ret |= FLAG_SERVICE_NOT_MESSAGE;

        Ok(ret)
    }
    
    fn tx_make_can_id(&self, transfer_data: &CyphalTxPacketFrame, local_node_id: u8) -> Result<u32, Box<dyn std::error::Error>> {
        let pl_mtu: u8 = self.tx_get_presentation_layer_mtu();

        let mut out: u32 = 0;

        if transfer_data.props.transfer_kind == CyphalTransferKind::Message &&
            transfer_data.props.remote_node_id ==  CYPHAL_NODE_ID_UNSET &&
            transfer_data.props.port_id <= CYPHAL_SUBJECT_ID_MAX
        {
            out = if local_node_id <= CYPHAL_NODE_ID_MAX {
                match Self::tx_make_message_session_specifier(transfer_data.props.port_id, local_node_id) {
                    Ok(x) => x,
                    Err(err) => return Err(err)
                }
            }
            else if transfer_data.payload_size <= pl_mtu as usize {
                let c: u8 = match crc_add(CRC_INITIAL, transfer_data.payload_size, &transfer_data.payload) {
                    Ok(x) => (x & CYPHAL_NODE_ID_MAX as u16) as u8,
                    Err(err) => return Err(err) 
                };
                match Self::tx_make_message_session_specifier(transfer_data.props.port_id, c) {
                    Ok(x) => x| FLAG_ANONYMOUS_MESSAGE,
                    Err(err) => return Err(err)
                }
            }
            else {
                // Anonymous multi-frame message trs are not allowed.
                return Err("INVALID_ARGUMENT".into());
            };
        } else if ((transfer_data.props.transfer_kind == CyphalTransferKind::Request) ||(transfer_data.props.transfer_kind == CyphalTransferKind::Response)) &&
            (transfer_data.props.remote_node_id <= CYPHAL_NODE_ID_MAX) &&
            (transfer_data.props.port_id <= CYPHAL_SERVICE_ID_MAX)
        {
            out = if local_node_id <= CYPHAL_NODE_ID_MAX {
                match Self::tx_make_service_session_specifier(
                    transfer_data.props.port_id,
                    transfer_data.props.transfer_kind == CyphalTransferKind::Request,
                    local_node_id,
                    transfer_data.props.remote_node_id
                ) {
                    Ok(x) => x,
                    Err(err) => return Err(err)
                }
            }
            else {
                return Err("INVALID_ARGUMENT".into())  // Anonymous service transfers are not allowed.
            }
        }

        if out > 0 {
            Ok(out | ((transfer_data.props.priority as u32) << OFFSET_PRIORITY))
        } else {
            Err("INVALID ID".into())
        }
    }

    fn tx_round_frame_payload_sizeup(x: usize) -> Result<usize, Box<dyn std::error::Error>> {
        if x >= 65 {
            return Err("INVALID DATA FRAME".into());
        };
        Ok(CAN_DLC_TO_DLEN[CAN_DLEN_TO_DLC[x] as usize] as usize)
    }

    fn tx_make_tail_byte(start_of_transfer: bool, end_of_transfer: bool, toggle: bool, transfer_id: CyphalTransferID) -> Result<u8, Box<dyn std::error::Error>> {
        if start_of_transfer && toggle != INITIAL_TOGGLE_STATE {
            return Err("INVALID BYTE PARSING".into());
        }
        let mut ret: u8 = 0;
        ret |= if start_of_transfer { TAIL_START_OF_TRANSFER } else { 0 };
        ret |= if end_of_transfer { TAIL_END_OF_TRANSFER } else { 0 };
        ret |= if toggle { TAIL_TOGGLE } else { 0 };
        ret |= transfer_id & CYPHAL_TRANSFER_ID_MAX;

        Ok(ret)
    }
}
