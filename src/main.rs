use std::collections::BTreeMap;

struct Db {
    memtable: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
}

impl Db {
    fn new() -> Self {
        Self {
            memtable: BTreeMap::new(),
        }
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.memtable.insert(key, Some(value));
    }

    fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.memtable.get(key).and_then(|v| v.as_deref())
    }

    fn delete(&mut self, key: &[u8]) {
        self.memtable.insert(key.to_vec(), None);
    }
}

fn main() {
    let mut db = Db::new();
    db.set(b"chave".to_vec(), b"valor".to_vec());
    if let Some(value) = db.get(b"chave") {
        println!("Valor para 'chave': {}", String::from_utf8_lossy(value));
    } else {
        println!("Chave não encontrada");
    }
    db.delete(b"chave");
    if let Some(value) = db.get(b"chave") {
        println!("Valor para 'chave': {}", String::from_utf8_lossy(value));
    } else {
        println!("Chave não encontrada");
    }
}
