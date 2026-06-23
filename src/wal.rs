use std::{
    fs::{File, OpenOptions},
    io::{ErrorKind::UnexpectedEof, Read, Result, Seek, SeekFrom, Write},
};

pub struct Wal {
    file: File,
}

impl Wal {
    pub fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn append(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<()> {
        if let Some(value) = value {
            self.file.write_all(&[1u8])?;
            self.file.write_all(&(key.len() as u32).to_be_bytes())?;
            self.file.write_all(key)?;
            self.file.write_all(&(value.len() as u32).to_be_bytes())?;

            self.file.write_all(value)?;
        } else {
            self.file.write_all(&[2u8])?;
            self.file.write_all(&(key.len() as u32).to_be_bytes())?;
            self.file.write_all(key)?;
            self.file.write_all(&0u32.to_be_bytes())?;
        }

        self.file.flush()?;

        Ok(())
    }

    pub fn read_all(&mut self) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut entries = Vec::new();

        loop {
            let mut buf = [0u8; 1];
            match self.file.read_exact(&mut buf) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let tipo = buf[0];

            let mut len_buf = [0u8; 4];
            match self.file.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let key_len = u32::from_be_bytes(len_buf) as usize;

            let mut key = vec![0u8; key_len];
            match self.file.read_exact(&mut key) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let mut len_buf = [0u8; 4];
            match self.file.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let value_len = u32::from_be_bytes(len_buf) as usize;

            let mut value = vec![0u8; value_len];
            match self.file.read_exact(&mut value) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let value = if tipo == 1 { Some(value) } else { None };

            entries.push((key, value));
        }

        Ok(entries)
    }
}
