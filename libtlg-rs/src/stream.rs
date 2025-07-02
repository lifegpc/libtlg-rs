use std::io::Read;

pub trait ReadExt {
    fn read_u32(&mut self) -> std::io::Result<u32>;
    fn read_u8(&mut self) -> std::io::Result<u8>;
}

impl<R: Read> ReadExt for R {
    fn read_u32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}
