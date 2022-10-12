mod mshell;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Access the system shell over mavlink.
    Mshell(mshell::Options),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Mshell(options) => {
            mshell::run(options);
        }
    }
}
