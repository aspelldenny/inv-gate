use clap::{Parser, Subcommand};

mod checks;
mod gate;

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
    /// Orchestrator — parity port of golden/security-gate.sh --mechanical-only branch
    Gate {
        /// Run all mechanical invariants (required — gate without --all exits 2)
        #[arg(long, required = true)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum CheckCommand {
    /// INV-009 — hardcoded secrets scan (parity port of golden/check-hardcoded-secrets.py)
    Secrets,
    /// INV-010 — runtime secrets scan (parity port of golden/check-runtime-secrets.py)
    Runtime,
    /// INV-001 — docker-compose host-bind check (parity port of golden/check-port-bind.py)
    Port,
    /// Prisma schema-safety — destructive migration guard (parity port of golden/check-schema-safety.sh)
    Schema,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Commands::Check { check } => match check {
            CheckCommand::Secrets => checks::secrets::run(),
            CheckCommand::Runtime => checks::runtime::run(),
            CheckCommand::Port => checks::port::run(),
            CheckCommand::Schema => checks::schema::run(),
        },
        Commands::Gate { all: true } => gate::run(),
        Commands::Gate { all: false } => {
            // clap enforces `required = true` so this branch is unreachable,
            // but exhaustiveness requires it. golden/security-gate.sh:14 → exit 2.
            unreachable!("clap enforces --all required")
        }
    };
    std::process::exit(exit_code);
}
