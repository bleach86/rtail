mod args;
mod constants;
mod follow_file;
mod tail_bytes;
mod tail_file;
mod tail_file_by_offset;
mod write_std_out;

// Re-export modules
pub use args::Args;
pub use follow_file::follow_file_inotify;
pub use tail_bytes::tail_bytes;
pub use tail_file::tail_file;
pub use tail_file_by_offset::offset_tail;
pub use write_std_out::write_out;
