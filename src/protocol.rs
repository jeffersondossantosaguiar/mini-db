pub enum Command {
    Set { key: Vec<u8>, value: Vec<u8> },
    Get { key: Vec<u8> },
    Delete { key: Vec<u8> },
}

pub fn parse(line: &str) -> Result<Command, String> {
    let command: Vec<&str> = line.split_whitespace().collect();

    match command.as_slice() {
        ["SET", key, value] => Ok(Command::Set {
            key: key.as_bytes().to_vec(),
            value: value.as_bytes().to_vec(),
        }),
        ["GET", key] => Ok(Command::Get {
            key: key.as_bytes().to_vec(),
        }),
        ["DELETE", key] => Ok(Command::Delete {
            key: key.as_bytes().to_vec(),
        }),
        _ => Err("Invalid Command".to_string()),
    }
}
