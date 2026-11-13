use crate::buffer::Buffer;

pub const TAG_NIL: u8 = 0;
pub const TAG_ERR: u8 = 1;
pub const TAG_STR: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_DBL: u8 = 4;
pub const TAG_ARR: u8 = 5;

pub const ERR_UNKNOWN: u32 = 1;
pub const ERR_TOO_BIG: u32 = 2;

pub fn out_nil(buffer: &mut Buffer) {
    buffer.append(&[TAG_NIL]);
}

pub fn out_error(buffer: &mut Buffer, error_code: u32, error_message: &[u8]) {
    buffer.append(&[TAG_ERR]);
    buffer.append(&error_code.to_le_bytes());
    buffer.append(&(error_message.len() as u32).to_le_bytes());
    buffer.append(error_message);
}

pub fn out_int(buffer: &mut Buffer, value: i64) {
    buffer.append(&[TAG_INT]);
    buffer.append(&value.to_le_bytes());
}

pub fn out_str(buffer: &mut Buffer, value: &[u8]) {
    buffer.append(&[TAG_STR]);
    buffer.append(&(value.len() as u32).to_le_bytes());
    buffer.append(value);
}

pub fn out_dbl(buffer: &mut Buffer, value: f64) {
    buffer.append(&[TAG_DBL]);
    buffer.append(&value.to_le_bytes());
}

pub fn out_arr(buffer: &mut Buffer, n: u32) {
    buffer.append(&[TAG_ARR]);
    buffer.append(&n.to_le_bytes());
}
