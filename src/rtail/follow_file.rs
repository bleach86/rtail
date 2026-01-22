use nix::{errno::Errno, sys::signal, unistd::Pid};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind, event::ModifyKind};
use std::{
    fs::{File, Metadata},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    path::PathBuf,
    process::exit,
    thread,
    time::Duration,
};

pub struct FollowFile {
    pub file: File,
    pub reader: BufReader<File>,
    pub position: u64,
    pub starting_len: u64,
    pub last_line: String,
    pub line_terminator: char,
    pub file_path: std::path::PathBuf,
    pub follow_name: bool,
    pub terminate_after_pid: Option<i32>,
}

impl FollowFile {
    pub fn new(
        file_path: &PathBuf,
        zero_terminated: bool,
        follow_name: bool,
        terminate_after_pid: Option<i32>,
    ) -> Result<FollowFile, Box<dyn std::error::Error>> {
        let file: File = File::open(file_path)?;
        let starting_len = file.metadata()?.len();
        let position: u64 = starting_len;
        let line_terminator: char = if zero_terminated { '\0' } else { '\n' };
        let last_line: String = String::new();
        let file_path: PathBuf = file_path.to_path_buf();
        let reader = BufReader::new(file.try_clone()?);

        Ok(FollowFile {
            file,
            reader,
            position,
            starting_len,
            last_line,
            line_terminator,
            file_path,
            follow_name,
            terminate_after_pid,
        })
    }

    pub fn follow_file_inotify(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.starting_len > 0 {
            self.file.seek(SeekFrom::End(-1))?;
            let mut buffer = [0; 1];
            self.file.read_exact(&mut buffer)?;
            let last_char = buffer[0] as char;
            if last_char != self.line_terminator {
                println!();
            }
        }

        // Start tailing from the end
        self.file.seek(SeekFrom::Start(self.position))?;

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;

        let follow_path = if self.follow_name {
            self.file_path.parent().unwrap_or(Path::new("."))
        } else {
            self.file_path.as_path()
        };

        watcher.watch(follow_path, RecursiveMode::NonRecursive)?;

        // If terminate_after_pid is set, spawn a thread to monitor the process
        if let Some(pid) = self.terminate_after_pid {
            thread::spawn(move || {
                check_process_running(pid);
            });
        }

        loop {
            match rx.recv() {
                Ok(event_result) => {
                    match event_result {
                        Ok(event) => {
                            for path in &event.paths {
                                if path == &self.file_path {
                                    match event.kind {
                                        EventKind::Modify(ModifyKind::Name(_)) => {
                                            if self.follow_name {
                                                let metadata = self.file.metadata()?;

                                                // Re-open the file in case it was rotated
                                                match reopen_file_if_rotated(
                                                    &self.file_path,
                                                    &metadata,
                                                ) {
                                                    Ok(file_opt) => match file_opt {
                                                        Some(new_file) => {
                                                            println!(
                                                                "File rotated, reopening {:?}",
                                                                self.file_path
                                                            );
                                                            self.file = new_file;
                                                            self.position = 0;
                                                            self.reader = BufReader::new(
                                                                self.file.try_clone()?,
                                                            );
                                                        }
                                                        None => {
                                                            // No rotation detected, carry on
                                                        }
                                                    },
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Error reopening file {:?}: {}",
                                                            self.file_path, e
                                                        );
                                                        continue;
                                                    }
                                                };

                                                if let Err(e) = self.process_file_change() {
                                                    eprintln!(
                                                        "Error processing file change: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        EventKind::Modify(ModifyKind::Data(_)) => {
                                            if let Err(e) = self.process_file_change() {
                                                eprintln!("Error processing file change: {}", e);
                                            }
                                        }

                                        _ => continue,
                                    }
                                }
                            }
                        }
                        Err(_) => continue,
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }
    }

    fn process_file_change<'reader>(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_size = self.file.metadata()?.len();

        if current_size == 0 {
            return Ok(());
        }

        let res = self.handle_modify(current_size)?;

        if !res && current_size < self.starting_len {
            // File was truncated
            self.position = 0;
            self.file.seek(SeekFrom::Start(0))?;
            self.reader = BufReader::new(self.file.try_clone()?);

            self.handle_modify(current_size)?;
        }

        self.starting_len = current_size;
        Ok(())
    }

    fn handle_modify<'reader>(
        &mut self,
        current_size: u64,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let bytes_read = self.reader.read_to_end(&mut buffer)?;

        if bytes_read == 0 {
            // File truncated?
            if current_size < self.position {
                println!("*File truncated*");
                self.last_line.clear();
                return Ok(false);
            }

            // No new data
            return Ok(true);
        }

        self.position += bytes_read as u64;

        let chunk = String::from_utf8_lossy(&buffer);

        // Append to last_line (incomplete tracking)
        self.last_line.push_str(&chunk);

        // Print only the new bytes
        print!("{}", chunk);
        std::io::stdout().flush()?;

        // Check if last line is complete
        if self.last_line.ends_with(self.line_terminator) {
            self.last_line.clear();
        }

        Ok(true)
    }
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
    current_metadata: &Metadata,
) -> Result<Option<File>, Box<dyn std::error::Error>> {
    // Try to reopen the file and compare inode and device numbers
    // If they differ, the file was rotated, so return the new file handle
    // If the file does not exist, keep trying until it does

    let mut first_attempt = true;

    loop {
        match File::open(file_path) {
            Ok(f) => {
                let new_metadata = f.metadata()?;

                if new_metadata.ino() != current_metadata.ino()
                    || new_metadata.dev() != current_metadata.dev()
                {
                    return Ok(Some(f));
                }

                return Ok(None);
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
