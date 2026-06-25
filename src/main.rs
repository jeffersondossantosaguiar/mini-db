mod compaction;
mod sstable;
mod wal;

use crate::compaction::compact;
use crate::sstable::{SSTable, existing_sstable_id, next_sstable_id, sstable_path};
use crate::wal::Wal;
use std::thread;
use std::time::Duration;
use std::{collections::BTreeMap, io::Result};

const MEMTABLE_SIZE_LIMIT: usize = 10; // 10B to tests 

struct Db {
    memtable: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    wal: Wal,
    size: usize,
    next_sstable_id: u64,
}

impl Db {
    fn new() -> Result<Self> {
        let mut wal = Wal::new("wal.log")?;
        let mut memtable = BTreeMap::new();
        let mut size = 0;

        for (key, value) in wal.read_all()? {
            size += key.len() + value.as_ref().map_or(0, |v| v.len());
            memtable.insert(key, value);
        }

        Ok(Self {
            memtable,
            wal,
            size,
            next_sstable_id: next_sstable_id(),
        })
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let value_len = value.len();
        let key_len = key.len();

        self.wal.append(&key, Some(&value))?;
        let old = self.memtable.insert(key, Some(value));

        if old.is_none() {
            self.size += key_len + value_len;
        } else if let Some(Some(old_value)) = old {
            self.size = self.size - old_value.len() + value_len;
        }

        if self.size >= MEMTABLE_SIZE_LIMIT {
            self.flush()?;
        }

        Ok(())
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.memtable.get(key) {
            None => {
                for sstable_id in (0..self.next_sstable_id).rev() {
                    let sstable_path = sstable_path(sstable_id);
                    let sstable = SSTable::new(&sstable_path);

                    if let Ok(Some(value)) = sstable.read(key) {
                        return value;
                    }
                }
                None
            }
            Some(None) => None,
            Some(Some(v)) => Some(v.clone()),
        }
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.wal.append(key, None)?;
        let old = self.memtable.insert(key.to_vec(), None);

        if let Some(Some(old_value)) = old {
            self.size -= old_value.len();
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let sstable_id = self.next_sstable_id;
        self.next_sstable_id += 1;

        let sstable_path = sstable_path(sstable_id);
        let sstable = SSTable::new(&sstable_path);

        sstable.write(&self.memtable)?;

        self.memtable.clear();
        self.size = 0;

        self.wal.clear()?;

        let nex_sstable_id = next_sstable_id();
        if nex_sstable_id.is_multiple_of(4) {
            let sstable_ids = existing_sstable_id();
            compact(&sstable_ids, nex_sstable_id)?;

            self.next_sstable_id += 1;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut db = Db::new()?;

    db.set(b"chave".to_vec(), b"valor".to_vec())?;

    thread::sleep(Duration::from_secs(2));

    db.set(b"chave2".to_vec(), b"valor".to_vec())?;

    thread::sleep(Duration::from_secs(2));
    db.set(b"chave3".to_vec(), b"valor".to_vec())?;

    thread::sleep(Duration::from_secs(2));
    db.set(b"chav4".to_vec(), b"valor".to_vec())?;
    thread::sleep(Duration::from_secs(2));
    db.set(b"chav5".to_vec(), b"valor".to_vec())?;
    thread::sleep(Duration::from_secs(2));

    if let Some(value) = db.get(b"chave") {
        println!("Valor para 'chave': {}", String::from_utf8_lossy(&value));
    } else {
        println!("Chave não encontrada");
    }

    db.delete(b"chave")?;

    if let Some(value) = db.get(b"chave") {
        println!("Valor para 'chave': {}", String::from_utf8_lossy(&value));
    } else {
        println!("Chave não encontrada");
    }

    Ok(())
}
