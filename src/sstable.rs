use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

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
}
