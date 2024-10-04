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
	Run,
	Test,
	// filetype specific commands
	Build,
	Deploy,
}

#[derive(Clone, ValueEnum, Debug,)]
pub enum ProjectType {
	Rust,
	Cargo,
	RustNvimConfig,
	Markdown,
	Zenn,
	LuaNvimConfig,
	Lua,
	TypeScript,
	GAS,
	/// render editing / generated html file
	WebSite,
	TOML,
	C,
	CPP,
	Swift,
	Python,
}
