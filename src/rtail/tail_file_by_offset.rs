use memchr::memchr_iter;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::rtail::{constants::CHUNK_SIZE, write_out};

pub fn offset_tail(
    file: &mut File,
    start_line: u64,
    zero_terminated: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut line_count: u64 = 1;
    let line_terminator: u8 = if zero_terminated { b'\0' } else { b'\n' };
    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let mut start_offset: u64 = 0;

    // start_line 0f 1 or 0 means from the beginning, so no need to seek
    if start_line > 1 {
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break; // EOF
            }

            let current_pos = file.seek(SeekFrom::Current(0))?;

            // Collect all terminators in this chunk
            let terminators: Vec<usize> =
                memchr_iter(line_terminator, &buffer[..bytes_read]).collect();

            // Check if the N-th line terminator is inside this chunk
            let lines_needed: u64 = start_line.saturating_sub(line_count);

            if lines_needed == 0 {
                // start_line has already been reached, use current position
                // This should not happen due to the loop condition, but just in case
                start_offset = current_pos - bytes_read as u64;
                break;
            } else if lines_needed <= terminators.len() as u64 {
                // The N-th line terminator is in this chunk
                let line_pos = terminators[(lines_needed - 1) as usize]; // zero-based
                start_offset = current_pos - bytes_read as u64 + line_pos as u64 + 1;
                break;
            } else {
                // Not in this chunk, just increment line_count
                line_count += terminators.len() as u64;
            }
        }
    }

    write_out(file, start_offset)?;

    Ok(())
}
