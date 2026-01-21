use nix::{errno::Errno, sys::signal, unistd::Pid};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::os::unix::fs::MetadataExt;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
    process::exit,
    thread,
    time::Duration,
};

pub fn follow_file_inotify(
    file_path: &Path,
    zero_terminated: bool,
    terminate_after_pid: Option<i32>,
    follow_name: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file: File = File::open(file_path)?;
    let mut starting_len = file.metadata()?.len();
    let mut position: u64 = starting_len;
    let line_terminator: char = if zero_terminated { '\0' } else { '\n' };
    let mut last_line: String = String::new();

    if starting_len > 0 {
        file.seek(SeekFrom::End(-1))?;
        let mut buffer = [0; 1];
        file.read_exact(&mut buffer)?;
        let last_char = buffer[0] as char;
        if last_char != line_terminator {
            println!();
        }
    }

    // Start tailing from the end
    file.seek(SeekFrom::Start(position))?;

    let mut reader: BufReader<&File> = BufReader::new(&file);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;

    let follow_path = if follow_name {
        file_path.parent().unwrap_or(Path::new("."))
    } else {
        file_path
    };

    watcher.watch(follow_path, RecursiveMode::NonRecursive)?;

    // If terminate_after_pid is set, spawn a thread to monitor the process
    if let Some(pid) = terminate_after_pid {
        thread::spawn(move || {
            check_process_running(pid);
        });
    }

    loop {
        match rx.recv() {
            Ok(event_result) => {
                match event_result {
                    Ok(event) => match event.kind {
                        notify::event::EventKind::Modify(_) => {
                            for path in &event.paths {
                                if follow_name {
                                    if path.file_name() == file_path.file_name() {
                                        // Re-open the file in case it was rotated
                                        reopen_file_if_rotated(file_path, &mut file)?;
                                        // Recreate reader after reopening file
                                        reader = BufReader::new(&file);
                                    } else {
                                        continue;
                                    }
                                }

                                let current_size = file.metadata()?.len();

                                if current_size == 0 {
                                    continue;
                                }

                                match handle_modify(
                                    current_size,
                                    &mut position,
                                    &mut reader,
                                    line_terminator,
                                    &mut last_line,
                                ) {
                                    Ok(res) => {
                                        match res {
                                            true => {}
                                            false => {
                                                // File was truncated, reset reader
                                                if current_size < starting_len {
                                                    position = 0;
                                                    file.seek(SeekFrom::Start(0))?;
                                                    reader = BufReader::new(&file);
                                                    handle_modify(
                                                        current_size,
                                                        &mut position,
                                                        &mut reader,
                                                        line_terminator,
                                                        &mut last_line,
                                                    )?;
                                                }
                                            }
                                        }
                                        starting_len = current_size;
                                    }
                                    Err(e) => eprintln!("Error handling file modification: {}", e),
                                };
                            }
                        }
                        _ => continue,
                    },
                    Err(_) => continue,
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}

fn handle_modify<'reader>(
    current_size: u64,
    position: &mut u64,
    reader: &mut BufReader<&'reader File>,
    line_terminator: char,
    last_line: &mut String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let bytes_read = reader.read_to_end(&mut buffer)?;

    if bytes_read == 0 {
        // File truncated?
        if current_size < *position {
            last_line.clear();
            return Ok(false);
        }

        // No new data
        return Ok(true);
    }

    *position += bytes_read as u64;

    let chunk = String::from_utf8_lossy(&buffer);

    // Append to last_line (incomplete tracking)
    last_line.push_str(&chunk);

    // Print only the new bytes
    print!("{}", chunk);
    std::io::stdout().flush()?;

    // Check if last line is complete
    if last_line.ends_with(line_terminator) {
        last_line.clear();
    }

    Ok(true)
}

fn check_process_running(pid: i32) {
    loop {
        match is_process_running(pid) {
            true => thread::sleep(Duration::from_secs(1)),
            false => break,
        }
    }

    exit(0);
}

fn is_process_running(pid: i32) -> bool {
    match signal::kill(Pid::from_raw(pid), None) {
        Ok(_) => true,
        Err(Errno::ESRCH) => false,
        Err(_) => true,
    }
}

fn reopen_file_if_rotated(
    file_path: &Path,
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut first_attempt = true;

    // Check if inode has changed
    let current_metadata = file.metadata()?;

    loop {
        match File::open(file_path) {
            Ok(f) => {
                let new_metadata = f.metadata()?;

                if new_metadata.ino() != current_metadata.ino()
                    || new_metadata.dev() != current_metadata.dev()
                {
                    *file = f;

                    return Ok(());
                }

                return Ok(());
            }
            Err(_) => {
                if first_attempt {
                    println!("File {:?} not found", file_path);
                    first_attempt = false;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
