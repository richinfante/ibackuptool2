pub fn pack_u64(val: u64) -> [u8; 8] {
    return [
        ((val & 0xFF00000000000000) >> 4*14) as u8,
        ((val & 0x00FF000000000000) >> 4*12) as u8,
        ((val & 0x0000FF0000000000) >> 4*10) as u8,
        ((val & 0x000000FF00000000) >> 4*8 ) as u8,
        ((val & 0x00000000FF000000) >> 4*6 ) as u8,
        ((val & 0x0000000000FF0000) >> 4*4 ) as u8,
        ((val & 0x000000000000FF00) >> 4*2 ) as u8,
        ((val & 0x00000000000000FF) >> 0*0 ) as u8,
    ]
}

pub fn unpack_64_bit(val: &[u8]) -> Option<[u8; 8]> {
    if val.len() < 8 {
        None
    } else {
        Some([val[0], val[1], val[2], val[3], val[4], val[5], val[6], val[7]])
    }
}