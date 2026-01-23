use memchr::memrchr_iter;
use std::{fs::File, os::unix::fs::FileExt};

use crate::rtail::{constants::CHUNK_SIZE, write_out};

pub fn tail_file(
    file: &mut File,
    num_lines: u64,
    zero_terminated: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pos: u64 = file.metadata()?.len();
    let mut line_count: u64 = 0;
    let line_terminator: u8 = if zero_terminated { b'\0' } else { b'\n' };

    let first_byte = if pos > 0 {
        let mut buffer = [0; 1];
        file.read_exact_at(&mut buffer, pos - 1)?;
        buffer[0]
    } else {
        0
    };

    if first_byte != line_terminator {
        line_count += 1; // Account for the first line if it doesn't start with a terminator
    }

    if num_lines == 0 {
        // Nothing to print
        return Ok(());
    }

    let mut chunk_buffer: Vec<u8> = vec![0; CHUNK_SIZE as usize];

    let mut line_offset = 0;

    // Read the file backwards in chunks until we find the required number of lines
    // Determine the byte offset to start printing from
    while pos > 0 && line_count <= num_lines {
        let read_size = CHUNK_SIZE.min(pos);

        // Move position back by read_size bytes
        // Use saturating_sub to avoid underflow
        pos = pos.saturating_sub(read_size);

        file.read_exact_at(&mut chunk_buffer[..read_size as usize], pos)?;

        let terminators: Vec<usize> =
            memrchr_iter(line_terminator, &chunk_buffer[..read_size as usize]).collect();

        let lines_needed = num_lines.saturating_sub(line_count) + 1;

        if lines_needed <= terminators.len() as u64 {
            // N-th line terminator is inside this chunk
            let idx = terminators[(lines_needed - 1) as usize];
            line_offset = pos + idx as u64 + 1; // start after terminator
            break;
        } else {
            // Not in this chunk, just increment line_count
            line_count += terminators.len() as u64;
        }
    }

    // Write from line_offset to end of file
    write_out(file, line_offset)?;

    Ok(())
}
