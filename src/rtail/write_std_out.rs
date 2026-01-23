use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Stdout, Write},
};

pub fn write_out(file: &mut File, start_offset: u64) -> Result<(), Box<dyn std::error::Error>> {
    let file_size: u64 = file.metadata()?.len();
    file.seek(SeekFrom::Start(start_offset))?;
    let stdout: Stdout = std::io::stdout();
    let chunk_size: usize = 1024 * 1024; // Read in 8KB chunks

    // Calculate total bytes to read
    // Saturating subtraction to avoid underflow
    let mut bytes_remaining: u64 = file_size.saturating_sub(start_offset);
    let mut stdout_lock = stdout.lock();

    // Read and print the rest of the file from the start_offset
    while bytes_remaining > 0 {
        let read_size = chunk_size.min(bytes_remaining as usize);
        let mut chunk_buffer = vec![0; read_size];

        // Read exact number of bytes into the buffer
        file.read_exact(&mut chunk_buffer)?;

        // Print the chunk as UTF-8, replacing invalid sequences
        stdout_lock.write_all(&chunk_buffer)?;
        bytes_remaining -= read_size as u64;
    }

    Ok(())
}
