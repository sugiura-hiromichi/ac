use crate::execution_detail;
use crate::parser;
use crate::util;
use anyhow::anyhow;
use anyhow::Result;
use std::path::PathBuf;
use strum::IntoEnumIterator;

/// `ac`に渡された引数、実行された環境、設定ファイルを元に以下の役割を果たします
/// - `ac`に渡された引数が適切か確認し、問題があればユーザーと対話しながら修正する
/// - 上記の手順を通じてチェックされたデータを元に`ac`が対象とするプロジェクトを管理します
pub struct ProjectManager {
	cli:          execution_detail::Cli,
	work_dir:     PathBuf,
	project_root: PathBuf,
	config:       ProjectManagerConfig,
}

impl ProjectManager {
	pub fn init() -> Result<Self,> {
		let cur_dir = std::env::current_dir()?;
		let mut pm = Self {
			cli:          execution_detail::Cli::init(),
			work_dir:     cur_dir.clone(),
			project_root: cur_dir,
			config:       ProjectManagerConfig::load()?,
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
	/// - `cli.project_type`がNoneだった場合、プロジェクトの種類を検出して`cli.project_type`
	///   にセットします
	fn detect_project(&mut self,) -> Result<(),> {
		self.root_and_type()?;
		self.init_pick()
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
	/// # TODO
	///
	/// - プロジェクトルートを探す際に`.git/`を考慮する
	/// - 設定を反映する
	fn root_and_type(&mut self,) -> Result<(),> {
		use execution_detail::ProjectType::*;
		match self.cli.project_type {
			// ユーザーがプロジェクトタイプを指定した場合
			Some(ref pt,) => match pt {
				Rust => {
					if let Some(p,) = self.lookup("main.rs",)? {
						self.project_root = p;
					}
				},
				// TODO: add support for `rust-project.json`
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
				Just => {
					if let Some(p,) = self.lookup("justfile",)? {
						self.project_root = p;
					}
				},
				DotFiles => {
					self.project_root = match std::env::var("XDG_CONFIG_HOME",) {
						Ok(p,) => p,
						Err(_,) => match std::env::var("HOME",) {
							Ok(p,) => p,
							Err(_,) => {
								unimplemented!("set environment vairable HOME or XDG_CONFIG_HOME")
							},
						},
					}
					.into();
				},
				Scheme => todo!(),
				Lisp => todo!(),
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
				// NOTE: why don't we restrict makefile support only for `C/CPP`?
				Markdown | Lua | C | CPP | Swift | Python => (),
			},
			// ユーザーがプロジェクトタイプを指定しなかった場合
			None => {
				// PERF: `self.work_dir`にあるファイル、
				// フォルダの情報を元にある程度プロジェクトタイプを絞る
				for pt in execution_detail::ProjectType::iter() {
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
	/// # TODO
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

	fn lookdown(&self, target: &str,) -> Result<Vec<PathBuf,>,> {
		let mut rslt = vec![];
		let mut que = util::Queue::new();
		que.init(self.project_root.clone(),);

		while let Some(sub_dir,) = que.dequeue() {
			for entry in sub_dir.read_dir()? {
				let fpath = entry?.path();

				if fpath.is_dir() {
					que.enqueue(fpath,);
				} else if fpath.is_file() {
					if fpath.file_name().unwrap().to_str().unwrap().contains(target,) {
						rslt.push(fpath,);
					}
				} else {
					return Err(anyhow!(
						"failed to determine the path is dor or file. path may be broken symlink. \
						 fix problem"
					),);
				}
			}
		}

		Ok(rslt,)
	}

	/// この関数は決め打ちで、とりあえず最初にターゲットファイルであろうものを推測して`self.cli.
	/// target_file`に格納する関数です
	fn init_pick(&mut self,) -> Result<(),> {
		if self.cli.tarrget_file.is_none() {
			if let Some(target,) = self.cli.target_hint(None,) {
				let cands = self.lookdown(target,)?;
				if !cands.is_empty() {
					self.cli.tarrget_file = Some(cands[0].clone(),);
				}
			}
		}
		Ok((),)
	}

	/// # 前提
	///
	/// この関数は、`self.project_root`と`self.cli.project_type`が正しく検出されている
	/// ことを前提とします</br>
	///
	/// # Panic
	///
	/// ```rust
	/// assert!(self.cli.project_type.is_some());
	/// ```
	///
	/// # TODO
	///
	/// - プログラムのエントリーポイント（コマンドによっては開きたいファイル）を検出する
	///
	/// # FIX
	///
	/// - 検索の条件として引数`opts`を受け取り、条件に合うファイルのパスを返す
	fn target_file(&mut self,) -> Result<(),> {
		use execution_detail::ProjectType::*;
		assert!(self.cli.project_type.is_some());

		if self.cli.tarrget_file.is_none() {
			self.cli.tarrget_file = match self.cli.project_type.as_ref().unwrap() {
				RustNvimConfig => {
					let mut cargo_toml = self.project_root.clone();
					cargo_toml.push("Cargo.toml",);
					let config = parser::toml::des_toml(&cargo_toml,)?;
					match config.get("package",).unwrap() {
						toml::Value::Table(pkg,) => match pkg.get("name",).unwrap() {
							toml::Value::String(pkg_name,) => Some(PathBuf::from(pkg_name,),),
							_ => unreachable!("package name should be declared in Cargo.toml"),
						},
						_ => unreachable!("package table should be exist on Cargo.toml"),
					}
				},
				// `cargo`コマンドがエントリポイントを検出するので何もしない
				Cargo => None,
				Rust => {
					use crate::parser::rust;
					let sources = self.lookdown(".rs",)?;
					let mut rslt = None;
					for fp in sources {
						let ast = rust::get_rs_ast(fp.to_str().unwrap(),)?;
						if rust::get_fn(&ast, "main",).is_some() {
							rslt = Some(fp,);
						}
					}

					if rslt.is_none() {
						return Err(anyhow!(
							"project_type: Rust requires target_file, but `ac` could not \
							 determine target_file"
						),);
					}
					rslt
				},
				Just => todo!(),
				DotFiles => todo!(),
				Scheme | Lisp | Lua | TypeScript | C | CPP | Swift | Python => {
					use execution_detail::Command::*;
					let target = match self.cli.command.as_ref().unwrap() {
						Run => todo!(),
						Test => todo!(),
						Fix => todo!(),
						Init => todo!(),
						New => todo!(),
						Build => todo!(),
						Upload => todo!(),
						Open => todo!(),
						Config => todo!(),
						Install => todo!(),
					};
					Some(self.lookdown("main",)?[0],)
				},
				// tarrget_file is required by open command. open by title, not slug
				Zenn => todo!(),
				Markdown => todo!(),
				LuaNvimConfig => todo!(),
				GAS => todo!(),
				WebSite => Some(self.lookdown("index.html",)?[0].clone(),),
				C => todo!(),
				CPP => todo!(),
				Swift => todo!(),
				Python => todo!(),
			}
		}
		Ok((),)
	}
}

/// ---
/// この構造体は初期化時に設定ファイルのエラーをチェックし、問題なければロードします
/// TODO: `ProjectManagerConfig`から`ProjectManagerPlugin`に改名し、設定をプラグインとして読み込む
/// サードパーティのプラグインのサポートを追加する
pub struct ProjectManagerConfig {}

impl ProjectManagerConfig {
	pub fn load() -> Result<Self,> { todo!() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn path_representation_of_dir() -> Result<(),> {
		let cur_dir = std::env::current_dir()?;
		assert_eq!(cur_dir.to_str().unwrap(), "/Users/a/Downloads/rust/ac");
		assert!(cur_dir.is_dir());
		Ok((),)
	}
}
