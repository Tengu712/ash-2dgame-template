use png::{Decoder, DecodingError};
use std::io::{Cursor, Error, ErrorKind};

/// PNGデータをデコードする関数
pub fn decode_png(data: &'static [u8]) -> Result<(Vec<u8>, u32, u32), DecodingError> {
    let decoder = Decoder::new(Cursor::new(data));
    let mut reader = decoder.read_info()?;
    let size = reader
        .output_buffer_size()
        .ok_or(DecodingError::IoError(Error::new(
            ErrorKind::InvalidData,
            "failed to get output buffer size",
        )))?;
    let mut buf = vec![0; size];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    Ok((buf, info.width, info.height))
}
