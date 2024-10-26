//! this module provides cli input parse tool

//use anyhow::anyhow;
//use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

#[derive(Parser,)]
#[command(version, about)]
pub struct Cli {
	#[command(subcommand)]
	/// command to specify pm behivor. like run, test ...
	pub command: Option<Command,>,

	#[arg(short, long)]
	/// arguments passed to original command
	pub args_passed_to_original: Vec<String,>,

	#[arg(short, long)]
	pub project_type: Option<ProjectType,>,

	#[arg(short, long)]
	pub tarrget_file: Option<std::path::PathBuf,>,
}

impl Cli {
	pub fn init() -> Self {
		let mut cli = Cli::parse();
		if cli.command.is_none() {
			cli.command = Some(Command::Run,);
		}
		cli
	}

	/// この関数は`crate::project::ProjectManager::target_file`内で使われる事を想定しています
	/// `self.project_type`と`self.command`から、どのファイル名を探すべきかを返します
	///
	/// # Return
	///
	/// `self.project_type`がCargoの場合など、ターゲットファイルが必要ないケースでは`None`を返します
	///
	/// # TODO
	///
	/// - `opt`を使用する
	pub fn target_hint(&self, opt: Option<&str,>,) -> Option<&str,> {
		assert!(self.project_type.is_some());
		assert!(self.command.is_some());

		use Command::*;
		use ProjectType::*;
		match self.project_type.as_ref().unwrap() {
			&RustNvimConfig | &Cargo | &Markdown | &GAS | &LuaNvimConfig | &Zenn | &DotFiles => {
				None
			},
			&Rust | &Scheme | &Lisp | &Lua | &TypeScript | &C | &CPP | &Swift | &Python => {
				self.command.as_ref().unwrap().default_target()
			},
			WebSite => Some("index.html",),
			Just => Some("justfile",),
		}
	}
}

#[derive(Subcommand,)]
pub enum Command {
	// general commands
	/// For markup language, this command will preview target,
	/// In other cases, this command will build & run executable. if package manager like
	/// `cargo` already exist, this will follow its way.
	Run,
	Test,
	Fix,
	Init,
	// makefile support
	/// TODO: currently `pm` only support creating a new file. add feature of creating new project
	New,
	// filetype specific commands
	Build,
	Upload,
	Open,
	// open config file e.g. Cargo.toml
	Config,
	Install,
}

impl Command {
	fn default_target(&self,) -> Option<&str,> {
		use Command::*;
		match self {
			&Run | &Build => Some("main",),
			&Test => Some("test",),
			_ => None,
		}
	}
}

#[derive(Clone, ValueEnum, Debug, strum_macros::EnumIter,)]
pub enum ProjectType {
	RustNvimConfig,
	Cargo,
	Rust,
	Just,
	Scheme,
	Lisp,
	// TODO: Add Node support
	Zenn,
	Markdown,
	DotFiles,
	// assumes executed within neovim
	LuaNvimConfig,
	Lua,
	TypeScript,
	GAS,
	/// render editing / generated html file
	WebSite,
	C,
	CPP,
	Swift,
	Python,
}

impl ProjectType {
	pub fn valid_commands(&self,) -> Vec<Command,> { todo!() }

	pub fn binary(&self,) -> &str {
		use ProjectType::*;
		match self {
			RustNvimConfig | Cargo => "cargo",
			Rust => "rustc",
			Just => "just",
			DotFiles => todo!(),
			Scheme => "chibi-scheme",
			Lisp => "sbcl",
			Zenn => "zenn",
			Markdown => todo!(),
			LuaNvimConfig => todo!(),
			Lua => "luajit",
			TypeScript => "tsx",
			GAS => "clasp",
			WebSite => todo!(),
			C => "clang",
			CPP => "clang++",
			Swift => "swiftc",
			Python => "python3",
		}
	}
}
