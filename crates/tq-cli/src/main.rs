mod cli;
mod error;

use clap::Parser;
use cli::{CheckArgs, Cli, Command};
use error::{CliError, Result};

fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Check(args) => run_check(&args),
    }
}

fn run_check(args: &CheckArgs) -> Result<()> {
    if let Some(config_path) = &args.config {
        if args.isolated {
            return Err(CliError::from_isolated_with_config(config_path));
        }

        if !config_path.exists() {
            return Err(CliError::from_missing_config(config_path));
        }

        if !config_path.is_file() {
            return Err(CliError::from_non_file_config(config_path));
        }
    }

    Ok(())
}
