use super::defines::*;

pub fn crc_add_byte(crc: u16, byte: u8) -> u16 {
    const TOP: TransferCRC = 0x8000;
    const POLY: TransferCRC = 0x1021;
    let mut out: TransferCRC  = crc ^ ((byte as u16) << BITS_PER_BYTE) as u16;
    // Consider adding a compilation option that replaces this with a CRC table. Adds 512 bytes of ROM.
    // Do not fold this into a loop because a size-optimizing compiler won't unroll it degrading the performance.
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out = (((out << 1) as u16) ^ (if (out & TOP) != 0 { POLY } else  { 0 })) as u16;
    out
}

pub fn crc_add(crc: u16, size: usize, data: &[u8]) -> Result<u16, Box<dyn std::error::Error>>{
    if data.len() != size {
        return Err("INVALID DATA LENGTH".into());
    };

    let mut out: u16 = crc;
    for v in data {
        out = crc_add_byte(out, *v);
    }
    Ok(out)
}
