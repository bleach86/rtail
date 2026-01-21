use clap::Parser;
use std::{
    env::home_dir,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

mod rtail;
use rtail::{Args, follow_file_inotify, offset_tail, tail_bytes, tail_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    // Collect inputfiles, expanding ~ to home directory if needed
    let input_files = args
        .filename
        .clone()
        .unwrap_or(vec!["stdin".to_string()])
        .into_iter()
        .map(|in_file| {
            if in_file.starts_with('~') {
                if let Some(home_path) = home_dir() {
                    return in_file.replacen('~', &home_path.to_string_lossy(), 1);
                }
            }
            in_file
        })
        .collect::<Vec<String>>();

    // Parse num_lines argument
    let mut is_plus_lines: bool = false;
    let num_lines: u64 = if args.num_lines.starts_with('+') {
        let n = args.num_lines[1..].parse::<u64>()?;
        is_plus_lines = true;
        n
    } else {
        match args.num_lines.parse::<u64>() {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error parsing number of lines '{}': {}", args.num_lines, e);
                return Ok(());
            }
        }
    };

    // Parse bytes argument if provided
    let mut is_plus_bytes: bool = false;
    let num_bytes: Option<u64> = match &args.bytes {
        Some(byte_str) => {
            if byte_str.starts_with('+') {
                is_plus_bytes = true;
            }
            match byte_str.trim_start_matches('+').parse::<u64>() {
                Ok(n) => Some(n),
                Err(e) => {
                    eprintln!("Error parsing number of bytes '{}': {}", byte_str, e);
                    return Ok(());
                }
            }
        }
        None => None,
    };

    // Process each input file
    for input_file in input_files.clone() {
        // Open the file
        let mut file: File = if input_file == "stdin" {
            // Read from stdin
            let stdin = std::io::stdin();
            let handle = stdin.lock();
            let temp_file_path = "/tmp/rtail_temp_file.txt";

            // Write stdin to a temporary file
            let mut temp_file = File::create(temp_file_path)?;
            std::io::copy(&mut handle.take(10 * 1024 * 1024), &mut temp_file)?;
            File::open(temp_file_path)?
        } else {
            match File::open(&input_file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error opening file '{}' for reading: {}", input_file, e);
                    continue;
                }
            }
        };

        // Print header if multiple files or verbose
        if (input_files.len() > 1 && input_file != "stdin" || args.verbose) && !args.quiet {
            let pre_new_line: &str = if input_file == input_files[0] {
                ""
            } else {
                "\n"
            };
            println!("{}==> {} <==", pre_new_line, input_file);
        }

        // Call appropriate tail function
        match num_bytes {
            Some(n) => {
                // Tail by bytes
                tail_bytes(&mut file, n, is_plus_bytes)?;
            }
            None => {
                if is_plus_lines {
                    // Tail by offset from line N
                    offset_tail(&mut file, num_lines, args.zero_terminated)?;
                } else {
                    // Tail by last N lines
                    tail_file(&mut file, num_lines, args.zero_terminated)?;
                }
            }
        }
    }

    // Handle follow option
    if args.follow || args.follow_name {
        // Only follow a single file
        if input_files.len() > 1 {
            println!();
            eprintln!("Error: --follow option can only be used with a single file.");
            std::process::exit(1);
        } else if input_files[0] == "stdin" {
            println!();
            eprintln!("Error: --follow option cannot be used with stdin.");
            std::process::exit(1);
        }
        // Follow the specified file
        let follow_file_name: String = input_files[0].clone();
        let follow_full_path: PathBuf = Path::new(&follow_file_name).canonicalize()?;
        follow_file_inotify(
            &follow_full_path,
            args.zero_terminated,
            args.terminate_after_pid,
            args.follow_name,
        )?;
    }

    Ok(())
}
