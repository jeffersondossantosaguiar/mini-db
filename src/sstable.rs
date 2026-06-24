use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::fs::read_dir;
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Read, Result, Seek, SeekFrom, Write};

const SSTABLE_NAME_PREFIX: &str = "sstable_";
const SSTABLE_EXTENSION: &str = ".sst";

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

pub fn sstable_path(id: u64) -> String {
    format!("{}{}{}", SSTABLE_NAME_PREFIX, id, SSTABLE_EXTENSION)
}

pub fn next_sstable_id() -> u64 {
    let mut id = 0;

    if let Ok(files) = read_dir(".") {
        for file in files.flatten() {
            let file_name = file.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with(SSTABLE_NAME_PREFIX)
                && file_name_str.ends_with(SSTABLE_EXTENSION)
                && let Some(id_str) = file_name_str
                    .strip_prefix(SSTABLE_NAME_PREFIX)
                    .and_then(|s| s.strip_suffix(SSTABLE_EXTENSION))
                && let Ok(file_id) = id_str.parse::<u64>()
                && file_id >= id
            {
                id = file_id + 1;
            };
        }
    };
    id
}
