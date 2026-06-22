use std::{
    fs::{File, OpenOptions},
    io::{Result, Write},
};

struct Wal {
    file: File,
}

impl Wal {
    fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self { file })
    }

    fn append(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<()> {
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
}
