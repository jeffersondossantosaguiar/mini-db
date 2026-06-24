use std::fs::read_dir;

pub fn next_sstable_id() -> u64 {
    let mut id = 0;

    if let Ok(files) = read_dir(".") {
        for file in files.flatten() {
            let file_name = file.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with("sstable_")
                && file_name_str.ends_with(".dat")
                && let Some(id_str) = file_name_str
                    .strip_prefix("sstable_")
                    .and_then(|s| s.strip_suffix(".dat"))
                && let Ok(file_id) = id_str.parse::<u64>()
                && file_id >= id
            {
                id = file_id + 1;
            };
        }
    };
    id
}
