use std::fs::{File, OpenOptions};
use chrono::Local;
use std::io::Write;

#[derive(Debug, PartialEq, strum::EnumString, strum::Display)]
pub enum LogType {
	INFO,
	ERROR,

}

pub struct Log {
	file: File,
}

impl Log {
	pub fn open(file_path: &str) -> Self {
		let log_file = match OpenOptions::new()
			.append(true)
			.create(true)
			.open(file_path) {
			Ok(file) => file,
			Err(e) => {
				panic!("{}", format!("ログファイルを開けませんでした:{}", e));
			}
		};

		Log { file: log_file }
	}

	pub fn write(&mut self, log_type: LogType, message: &str) {
		writeln!(self.file, "{}", format!("{} [{}]: {}", Local::now(), log_type.to_string(), message)).unwrap();
	}
}