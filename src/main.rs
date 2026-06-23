mod wal;

use crate::wal::Wal;
use std::{collections::BTreeMap, io::Result};

struct Db {
    memtable: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    wal: Wal,
}

impl Db {
    fn new() -> Result<Self> {
        let mut wal = Wal::new("wal.log")?;
        let mut memtable = BTreeMap::new();

        for (key, value) in wal.read_all()? {
            memtable.insert(key, value);
        }

        Ok(Self { memtable, wal })
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.wal.append(&key, Some(&value))?;
        self.memtable.insert(key, Some(value));
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.memtable.get(key).and_then(|v| v.as_deref())
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.wal.append(&key, None)?;
        self.memtable.insert(key.to_vec(), None);
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut db = Db::new()?;

    db.set(b"chave".to_vec(), b"valor".to_vec())?;

    if let Some(value) = db.get(b"chave") {
        println!("Valor para 'chave': {}", String::from_utf8_lossy(value));
    } else {
        println!("Chave não encontrada");
    }

    db.delete(b"chave")?;

    if let Some(value) = db.get(b"chave") {
        println!("Valor para 'chave': {}", String::from_utf8_lossy(value));
    } else {
        println!("Chave não encontrada");
    }

    Ok(())
}
