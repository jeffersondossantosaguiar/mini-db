use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Read, Result, Seek, SeekFrom, Write};

pub struct SSTable {
    path: String,
}

impl SSTable {
    pub fn new(path: &str) -> Self {
        SSTable {
            path: path.to_string(),
        }
    }

    pub fn write(&self, memtable: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        for (key, value) in memtable {
            file.write_all(&(key.len() as u32).to_be_bytes())?;
            file.write_all(key)?;

            match value {
                Some(v) => {
                    file.write_all(&(v.len() as u32).to_be_bytes())?;
                    file.write_all(v)?;
                }
                None => {
                    file.write_all(&0u32.to_be_bytes())?;
                }
            }
        }

        file.flush()?;
        Ok(())
    }

    pub fn read(&self, key: &[u8]) -> Result<Option<Option<Vec<u8>>>> {
        let mut file = OpenOptions::new().read(true).open(&self.path)?;

        file.seek(SeekFrom::Start(0))?;

        loop {
            let mut key_len_buf = [0u8; 4];
            match file.read_exact(&mut key_len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let key_len = u32::from_be_bytes(key_len_buf) as usize;
            let mut key_buf = vec![0u8; key_len];
            match file.read_exact(&mut key_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let mut value_len_buf = [0u8; 4];
            match file.read_exact(&mut value_len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let value_len = u32::from_be_bytes(value_len_buf) as usize;

            if key_buf.as_slice() < key {
                file.seek(SeekFrom::Current(value_len as i64))?;
                continue;
            } else if key_buf == key {
                if value_len > 0 {
                    let mut value_buf = vec![0u8; value_len];
                    match file.read_exact(&mut value_buf) {
                        Ok(_) => {}
                        Err(e) if e.kind() == UnexpectedEof => break,
                        Err(e) => return Err(e),
                    }
                    return Ok(Some(Some(value_buf)));
                } else {
                    return Ok(Some(None));
                }
            } else {
                break;
            }
        }

        Ok(None)
    }
}
