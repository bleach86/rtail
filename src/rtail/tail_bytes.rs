use std::fs::File;

use crate::rtail::write_out;

pub fn tail_bytes(
    file: &mut File,
    num_bytes: u64,
    is_plus: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_size: u64 = file.metadata()?.len();
    let start_pos: u64 = if num_bytes > file_size {
        0
    } else {
        if is_plus {
            num_bytes
        } else {
            file_size - num_bytes
        }
    };

    // Read and print the rest of the file from the start_offset
    write_out(file, start_pos)?;

    Ok(())
}
