use std::env;

use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Print the last part of files to standard output.";
const USAGE: &str = "\n  rtail [OPTIONS] [FILENAME]...\n";
const EXAMPLES: &str =
    "\n\x1b[1;4mEXAMPLES:\x1b[0m\n  rtail -n 20 file.txt\n  rtail -f -n +10 logfile.log\n";

#[derive(Parser, Debug)]
#[command(author = AUTHOR, version = VERSION, about = ABOUT,
    override_usage = format!("{}{}", USAGE, EXAMPLES))]
pub struct Args {
    pub filename: Option<Vec<String>>,

    /// Output the last NUM lines;
    /// or use -n +NUM to output starting with line NUM of each file
    #[arg(short, long = "lines", default_value_t = String::from("10"))]
    pub num_lines: String,

    /// Output the last NUM bytes;
    /// or use -c +NUM to output starting with byte NUM of each file
    #[arg(short = 'c', long)]
    pub bytes: Option<String>,

    /// Follow the file for new lines
    #[arg(short, long, default_value_t = false)]
    pub follow: bool,

    /// Use with -f, terminate after process ID, PID dies
    #[arg(long = "pid", requires = "follow")]
    pub terminate_after_pid: Option<i32>,

    /// Follow by file name (handle log rotation)
    #[arg(long, default_value_t = false)]
    pub follow_name: bool,

    /// The line delimiter is NUL, not newline
    #[arg(short = 'z', long, default_value_t = false)]
    pub zero_terminated: bool,

    /// Always output headers giving file names
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,

    /// Do not output headers giving file names
    #[arg(short = 'q', long = "quiet", alias = "silent", default_value_t = false)]
    pub quiet: bool,
}
