#![feature(pattern, never_type, file_buffered)]
//! # TODO: AI機能の追加

use anyhow::anyhow;
use anyhow::Result;
use cli::ProjectType;
use project::ProjectManager;
use std::path::PathBuf;

mod cli;
mod project;
mod serde_type;

/// 実際のコマンドの実行とエラー処理はこの`main`関数で行われます
fn main() -> Result<(),> {
	let pm = ProjectManager::init()?;

	Err(anyhow!("this project is still imcomplete"),)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn read_dir_test() -> Result<(),> {
		let dir = std::env::current_dir()?.read_dir()?;
		for entry in dir {
			println!("{}", entry?.path().display());
		}
		Ok((),)
	}
}
