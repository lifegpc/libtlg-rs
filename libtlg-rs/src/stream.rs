use std::io::Read;
#[cfg(feature = "encode")]
use std::io::Write;

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

#[cfg(feature = "encode")]
pub trait WriteExt {
    fn write_u32(&mut self, value: u32) -> std::io::Result<()>;
    fn write_u8(&mut self, value: u8) -> std::io::Result<()>;
}

#[cfg(feature = "encode")]
impl<W: Write> WriteExt for W {
    fn write_u32(&mut self, value: u32) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_u8(&mut self, value: u8) -> std::io::Result<()> {
        self.write_all(&[value])
    }
}
