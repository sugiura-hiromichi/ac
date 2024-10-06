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
	/// TODO: currently `pm` only support creating a new file. add feature of creating new project
	New,
	// filetype specific commands
	/// For compiled language
	Build,
	Deploy,
}

#[derive(Clone, ValueEnum, Debug, strum_macros::EnumIter,)]
pub enum ProjectType {
	RustNvimConfig,
	Cargo,
	Rust,
	Zenn,
	Markdown,
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
