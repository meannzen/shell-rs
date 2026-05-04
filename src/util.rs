use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, Write},
};

use crate::error::ShellError;

pub fn read_history(path: &str) -> Vec<String> {
    File::open(path)
        .map(|f| BufReader::new(f).lines().map_while(Result::ok).collect())
        .unwrap_or_default()
}

pub fn write_history(path: &str, histories: &[String]) -> Result<(), ShellError> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    for line in histories {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

pub fn append_history(
    path: &str,
    histories: &[String],
    from_index: usize,
) -> Result<usize, ShellError> {
    let write_start = match File::open(path) {
        Ok(mut f) => {
            let len = f.seek(io::SeekFrom::End(0)).unwrap_or(0);
            if len >= 2 {
                let _ = f.seek(io::SeekFrom::End(-2));
                let mut buf = [0u8; 2];
                if f.read_exact(&mut buf).is_ok() && buf == [b'\n', b'\n'] {
                    len - 1
                } else {
                    len
                }
            } else {
                len
            }
        }
        Err(_) => 0,
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.seek(io::SeekFrom::Start(write_start))?;
    for line in histories.iter().skip(from_index) {
        writeln!(file, "{}", line)?;
    }
    Ok(histories.len())
}
