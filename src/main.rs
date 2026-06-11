use clap::{Parser, Subcommand};

mod checks;

#[derive(Parser)]
#[command(name = "inv-gate", version, about = "Mechanical security invariant checks (INV-009...) — CLI + MCP dual mode")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mechanical INV checks
    Check {
        #[command(subcommand)]
        check: CheckCommand,
    },
}

#[derive(Subcommand)]
enum CheckCommand {
    /// INV-009 — hardcoded secrets scan (parity port of golden/check-hardcoded-secrets.py)
    Secrets,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Commands::Check { check } => match check {
            CheckCommand::Secrets => checks::secrets::run(),
        },
    };
    std::process::exit(exit_code);
}
