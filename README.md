# mini-db

A key-value storage engine built from scratch in Rust, implementing an LSM-Tree architecture.

## Architecture

- **WAL (Write-Ahead Log)** — append-only log for durability
- **Memtable** — in-memory sorted structure (`BTreeMap`) for fast reads/writes
- **SSTables** — immutable on-disk sorted files
- **Compaction** — merges SSTables to reclaim space

## Status

- [x] Phase 1: Memtable (in-memory `SET`, `GET`, `DELETE` with tombstone support)
- [x] Phase 1: WAL (durability across restarts)
- [x] Phase 2: SSTable flush
- [x] Phase 3: Compaction
- [ ] Phase 4: TCP server

## TODO

- Escrita em streaming durante compaction (O(K) memória em vez de O(N))
- Arquivo temporário + rename atômico para proteção contra crash
