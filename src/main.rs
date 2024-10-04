#![feature(pattern)]
#![allow(unused_variables, unused_imports, dead_code)]
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use clap::Parser;
use clap::ValueEnum;
use cli::Cli;
use cli::ProjectType;
use std::path::Path;
use std::path::PathBuf;

mod cli;

/// 保持している、対象となるプロジェクトについての情報を元に、
/// コマンドの適切な実行を推論する役割を担います
struct ProjectManager {
	cli:          cli::Cli,
	work_dir:     PathBuf,
	project_root: PathBuf,
}

impl ProjectManager {
	fn init() -> Result<Self,> {
		let cur_dir = std::env::current_dir()?;
		//let cur_dir = cur_dir.as_path();
		let mut pm = Self {
			cli:          cli::Cli::init(),
			work_dir:     cur_dir.clone(),
			project_root: cur_dir,
		};

		pm.detect_project()?;
		Ok(pm,)
	}

	/// この関数はコマンドが実行されているpathを必要とします
	/// このpathは`Cli`のコマンドラインから渡されたinputをparseする、
	/// という目的とは関係ないので`Cli`ではなく`ProjectManager`内にあります
	///
	/// # 動作
	///
	/// - `cli.project_type`がNoneだった場合、プロジェクトの種類を検出して`cli.
	/// project_type`にセットします
	/// - コマンドが実行されたpathを戻り値として返します
	/// - プロジェクトのルートディレクトリを戻り値として返します
	///
	/// # 戻り値
	///
	/// この関数は`anyhow::Result<(Option<cli::ProjectType,>,
	/// PathBuf, PathBuf)`型を返します
	/// エラーが起こった場合はErr()、
	/// 処理が正常に完了した場合は`(Some(project_type), work_dir)`
	/// をOk()にラップして返します
	fn detect_project(&mut self,) -> Result<(),> { todo!() }

	/// この関数はプロジェクトのルートディレクトリのpathを返します
	fn project_root(&mut self,) -> Result<(),> {
		use ProjectType::*;
		match self.cli.project_type {
			// ユーザーがプロジェクトタイプを指定した場合
			Some(ref pt,) => match pt {
				Rust => {
					if let Some(p,) = self.target_exist_upstream("main.rs",)? {
						self.project_root = p;
					}
				},
				Cargo => match self.target_exist_upstream("Cargo.toml",)? {
					None => {
						let e = Err(anyhow!(
							"specified project_type `{:?}` seems incorrect",
							self.cli.project_type.take().unwrap()
						),);
						return e;
					},
					Some(p,) => self.project_root = p,
				},
				RustNvimConfig => {
					let config_home = option_env!("XDG_CONFIG_HOME")
						.map_or(format!("{}/.config", env!("HOME")), |p| p.to_string(),);
					if !self.work_dir.to_str().unwrap().contains(&config_home,) {
						return Err(anyhow!(
							"current directory: {}\n seems not in `$HOME/.config` or \
							 `$XDG_CONFIG_HOME/`.\n if you are in correct path, consider checking \
							 $HOME or $XDG_CONFIG_HOME.",
							self.work_dir.display()
						),);
					}
					match self.target_exist_upstream("Cargo.toml",)? {
						Some(p,) => self.project_root = p,
						None => (),
					}
				},
				Markdown => todo!(),
				Zenn => todo!(),
				LuaNvimConfig => todo!(),
				Lua => todo!(),
				TypeScript => todo!(),
				GAS => todo!(),
				WebSite => todo!(),
				TOML => todo!(),
				C => todo!(),
				CPP => todo!(),
				Swift => todo!(),
				Python => todo!(),
			},
			// ユーザーがプロジェクトタイプを指定しなかった場合
			None => {
				// code which detect project root
				self.project_type()?;
				todo!()
			},
		};

		Ok((),)
	}

	fn project_type_miss(&mut self,) -> Result<!,> {
		Err(anyhow!(
			"specified project_type `{:?}` seems incorrect",
			self.cli.project_type.take().unwrap()
		),)
	}

	// `target`という名称のファイルまたはディレクトリを含みかつ、
	// コマンドが実行されているパスの上流でもあるようなパスを返します
	//
	// # Return
	//
	// 現在のパスが`$HOME`を含む場合 →
	// `$HOME`内に`target`を含むパスが存在しない場合`Ok(None)`を返します
	//
	// 現在のパスが`$HOME`を含まない場合 →
	// `/`内に`target`を含むパスが存在しない場合`Ok(None)`を返します
	fn target_exist_upstream(&self, target: &str,) -> Result<Option<PathBuf,>,> {
		let mut upper_path = self.work_dir.clone();

		for element in self.work_dir.iter().rev() {
			for entry in upper_path.read_dir()? {
				if entry?.file_name() == target {
					return Ok(Some(upper_path,),);
				}
			}

			let tmp = upper_path.pop();
			assert!(tmp);

			if upper_path.to_str().unwrap() == env!("HOME") {
				return Ok(None,);
			}
		}
		Ok(None,)
	}

	fn project_type(&mut self,) -> Result<(),> {
		let mut fts = vec![];

		// `work_dir`内のファイル・ディレクトリを走査します
		// 条件に合う拡張子が見つかった場合、`fts`に格納します
		for entry in self.work_dir.read_dir()? {
			let path = entry?.path();

			// `path`がファイルだった場合拡張子が有効なものであれば`fts`に追加
			if path.is_file() {
				let extention = path.extension();
				match extention {
					Some(ext,) => {
						let filetype = ext.to_str().filter(|&s| {
							s == "rs"
								|| s == "md" || s == "lua"
								|| s == "ts" || s == "tsx"
								|| s == "toml" || s == "c"
								|| s == "cpp" || s == "h"
								|| s == "html" || s == "swift"
								|| s == "py"
						},);
						if let Some(ft,) = filetype {
							fts.push(ft.to_owned(),);
						}
					},
					None =>
					// TODO: set filetype for special name file such like
					// `.zshenv`
					{
						()
					},
				}
			}
		}
		todo!()
	}

	fn target_file() -> Result<(),> { Ok((),) }
}

/// 実際のコマンドの実行とエラー処理はこの`main`関数で行われます
fn main() -> Result<(),> {
	let work_dir = std::env::current_dir()?;
	let pm = ProjectManager::init()?;

	Ok((),)
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
