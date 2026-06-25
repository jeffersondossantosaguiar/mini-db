use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
    fs::{File, OpenOptions, remove_file},
    io::{Read, Result, Seek, SeekFrom},
};

use crate::sstable::{SSTable, sstable_path};

struct SstableIterator {
    file: File,
    sstable_id: u64,
}

impl SstableIterator {
    pub fn new(path: &str, sstable_id: u64) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(path)?;

        file.seek(SeekFrom::Start(0))?;

        Ok(Self { file, sstable_id })
    }

    pub fn next(&mut self) -> Option<(Vec<u8>, Option<Vec<u8>>)> {
        let mut key_len_buf = [0u8; 4];
        match self.file.read_exact(&mut key_len_buf) {
            Ok(_) => {}
            Err(_) => return None,
        }

        let key_len = u32::from_be_bytes(key_len_buf) as usize;
        let mut key = vec![0u8; key_len];
        match self.file.read_exact(&mut key) {
            Ok(_) => {}
            Err(_) => return None,
        }

        let mut value_len_buf = [0u8; 4];
        match self.file.read_exact(&mut value_len_buf) {
            Ok(_) => {}
            Err(_) => return None,
        }

        let value_len = u32::from_be_bytes(value_len_buf) as usize;

        if value_len > 0 {
            let mut value = vec![0u8; value_len];
            match self.file.read_exact(&mut value) {
                Ok(_) => {}
                Err(_) => return None,
            }

            Some((key, Some(value)))
        } else {
            Some((key, None))
        }
    }
}

pub fn compact(sstable_ids: &[u64], new_id: u64) -> Result<()> {
    let mut iterators: Vec<SstableIterator> = Vec::new();

    for sstable_id in sstable_ids {
        let path = sstable_path(*sstable_id);

        iterators.push(SstableIterator::new(&path, *sstable_id)?);
    }

    let mut heap: BinaryHeap<Reverse<(Vec<u8>, u64, u64, Option<Vec<u8>>)>> = BinaryHeap::new();

    for (index, valor) in iterators.iter_mut().enumerate() {
        if let Some(entry) = valor.next() {
            heap.push(Reverse((entry.0, valor.sstable_id, index as u64, entry.1)));
        }
    }

    let mut result: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();

    while let Some(Reverse((key, sstable_id, index, value))) = heap.pop() {
        result.insert(key, value);

        if let Some(entry) = iterators[index as usize].next() {
            heap.push(Reverse((entry.0, sstable_id, index, entry.1)))
        }
    }

    let compacted: BTreeMap<Vec<u8>, Option<Vec<u8>>> = result
        .into_iter()
        .filter(|(_, value)| value.is_some())
        .collect();

    let path = sstable_path(new_id);

    let sstable = SSTable::new(&path);

    sstable.write(&compacted)?;

    for id in sstable_ids {
        let old_path = sstable_path(*id);

        remove_file(old_path)?;
    }

    Ok(())
}
