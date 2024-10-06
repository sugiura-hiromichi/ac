#![feature(pattern, never_type, file_buffered)]
use anyhow::anyhow;
use anyhow::Result;
use cli::ProjectType;
use std::path::PathBuf;
use strum::IntoEnumIterator;

mod cli;
mod serde_type;

/// `ac`に渡された引数、実行された環境、設定ファイルを元に以下の役割を果たします
/// - `ac`に渡された引数が適切か確認し、問題があればユーザーと対話しながら修正する
/// - 設定ファイルのエラーをチェックし、問題なければロードする
/// - 上記の手順を通じてチェックされたデータを元に`ac`が対象とするプロジェクト
struct ProjectManager {
	cli:          cli::Cli,
	work_dir:     PathBuf,
	project_root: PathBuf,
	config:       ProjectManagerConfig,
}

impl ProjectManager {
	fn init() -> Result<Self,> {
		let cur_dir = std::env::current_dir()?;
		let mut pm = Self {
			cli:          cli::Cli::init(),
			work_dir:     cur_dir.clone(),
			project_root: cur_dir,
			config:       Self::load_config()?,
		};

		pm.detect_project()?;
		Ok(pm,)
	}

	fn load_config() -> Result<ProjectManagerConfig,> { todo!() }

	/// この関数はコマンドが実行されているpathを必要とします
	/// このpathは`Cli`のコマンドラインから渡されたinputをparseする、
	/// という目的とは関係ないので`Cli`ではなく`ProjectManager`内にあります
	///
	/// # 動作
	///
	/// - `cli.project_type`がNoneだった場合、プロジェクトの種類を検出して`cli.project_type`
	///   にセットします
	fn detect_project(&mut self,) -> Result<(),> {
		self.root_and_type()?;
		self.target_file()
	}

	/// この関数はプロジェクトのルートディレクトリのpathを`self.project_root`にセットします
	/// その際に、`self.cli.project_type`が適切にセットされているか検証します
	///
	/// # Return
	///
	/// この関数は以下のケースでエラーを返します
	/// - ユーザーが指定したプロジェクトタイプがおかしい時
	/// - プロジェクトタイプを推測できない時
	///
	/// # TODO:
	///
	/// - プロジェクトルートを探す際に`.git/`を考慮する
	/// - 設定を反映する
	fn root_and_type(&mut self,) -> Result<(),> {
		use ProjectType::*;
		match self.cli.project_type {
			// ユーザーがプロジェクトタイプを指定した場合
			Some(ref pt,) => match pt {
				Rust => {
					if let Some(p,) = self.lookup("main.rs",)? {
						self.project_root = p;
					}
				},
				Cargo => match self.lookup("Cargo.toml",)? {
					Some(p,) => self.project_root = p,
					None => self.missed_project()?,
				},
				RustNvimConfig => {
					// TODO: adding support of configuration file then, load config home directory
					// from `self.config.config_home`
					// or automatically determine by given dotfile repository's url
					todo!("this project type is currently not supported");
					match self.lookup("Cargo.toml",)? {
						Some(p,) => self.project_root = p,
						None => self.missed_project()?,
					}
				},
				Zenn => match self.lookup("package.json",)? {
					Some(p,) => {
						let mut file = p.clone();
						file.push("package.json",);

						let pkg_jsn: serde_json::Value =
							serde_json::from_reader(std::fs::File::open_buffered(file,)?,)?;
						match pkg_jsn.get("dependencies",) {
							Some(v1,) if matches!(v1.get("zenn-cli"), Some(_)) => {
								self.project_root = p
							},
							_ => return Err(anyhow!("zenn-cli seems not installed locally"),),
						}
					},
					None => {
						// `zenn-cli`がグローバルにインストールされていた場合
						if std::process::Command::new("which",).arg("zenn",).output().is_ok() {
							let art_p = self.lookup("articles",)?;
							let book_p = self.lookup("books",)?;
							match (art_p, book_p,) {
								(Some(ap,), Some(bp,),) => {
									let ap_len = ap.components().count();
									let bp_len = bp.components().count();

									if ap_len > bp_len {
										self.project_root = ap;
									} else {
										self.project_root = bp;
									}
								},
								(_, Some(p,),) | (Some(p,), _,) => self.project_root = p,
								(None, None,) => self.missed_project()?,
							}
						} else {
							self.missed_project()?
						}
					},
				},
				LuaNvimConfig => {
					// RustNvimConfigとほぼ一緒
				},
				TypeScript => {
					if let Some(p,) = self.lookup("package.json",)? {
						self.project_root = p;
					}
				},
				GAS => match self.lookup("appscript.json",)? {
					Some(p,) => self.project_root = p,
					None => self.missed_project()?,
				},
				WebSite => match self.lookup("index.html",)? {
					Some(p,) => self.project_root = p,
					None => self.missed_project()?,
				},
				// TODO: `C/CPP`: makefile support
				Markdown | Lua | C | CPP | Swift | Python => (),
			},
			// ユーザーがプロジェクトタイプを指定しなかった場合
			None => {
				// PERF: `self.work_dir`にあるファイル、
				// フォルダの情報を元にある程度プロジェクトタイプを絞る
				for pt in ProjectType::iter() {
					self.cli.project_type = Some(pt,);
					if self.root_and_type().is_ok() {
						break;
					}
				}
			},
		};

		Ok((),)
	}

	/// ユーザーが指定したプロジェクトタイプが間違っていると考えられる場合に適切なエラーを投げる補助関数です
	///
	/// # TODO: ↓
	///
	/// プロジェクトタイプが間違っていた場合の処理として、ユーザーに
	/// 1. プロジェクトタイプを再入力してもらう
	/// 2. 指定されたプロジェクトを新たに作成する
	/// 3. コマンドを終了する
	/// 4. etc..
	/// というふうに選択肢を与える
	fn missed_project(&mut self,) -> Result<!,> {
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
	fn lookup(&self, target: &str,) -> Result<Option<PathBuf,>,> {
		let mut upper_path = self.work_dir.clone();

		loop {
			for entry in upper_path.read_dir()? {
				if entry?.file_name() == target {
					return Ok(Some(upper_path,),);
				}
			}

			if upper_path.to_str().unwrap() == env!("HOME") || !upper_path.pop() {
				break;
			}
		}
		Ok(None,)
	}

	fn target_file(&mut self,) -> Result<(),> { Ok((),) }
}

struct ProjectManagerConfig {}

/// 実際のコマンドの実行とエラー処理はこの`main`関数で行われます
fn main() -> Result<(),> {
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
