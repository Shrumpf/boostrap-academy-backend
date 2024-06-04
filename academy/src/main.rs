use std::path::PathBuf;

use academy::commands::{
    admin::AdminCommand, email::EmailCommand, jwt::JwtCommand, migrate::MigrateCommand,
    serve::serve, tasks::TaskCommand,
};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve => {
            let config = academy_config::load(&cli.config)?;
            serve(config).await?
        }
        Command::Migrate { command } => {
            let config = academy_config::load(&cli.config)?;
            command.invoke(config).await?
        }
        Command::Admin { command } => {
            let config = academy_config::load(&cli.config)?;
            command.invoke(config).await?
        }
        Command::Jwt { command } => {
            let config = academy_config::load(&cli.config)?;
            command.invoke(config).await?
        }
        Command::Email { command } => {
            let config = academy_config::load(&cli.config)?;
            command.invoke(config).await?
        }
        Command::Task { command } => {
            let config = academy_config::load(&cli.config)?;
            command.invoke(config).await?
        }
        Command::CheckConfig { verbose } => {
            let config = academy_config::load(&cli.config)?;
            if verbose {
                println!("{config:#?}");
            }
        }
        Command::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                env!("CARGO_BIN_NAME"),
                &mut std::io::stdout(),
            );
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long, env, default_value = "config.toml")]
    config: Vec<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the backend server
    #[command(aliases(["run", "start", "r", "s"]))]
    Serve,
    /// Run database migrations
    #[command(aliases(["mig", "m"]))]
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
    #[command(aliases(["a"]))]
    Admin {
        #[command(subcommand)]
        command: AdminCommand,
    },
    #[command(aliases(["j"]))]
    Jwt {
        #[command(subcommand)]
        command: JwtCommand,
    },
    #[command(aliases(["e"]))]
    Email {
        #[command(subcommand)]
        command: EmailCommand,
    },
    #[command(aliases(["t"]))]
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Validate config files
    CheckConfig {
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate shell completions
    Completion {
        /// The shell to generate completions for
        #[clap(value_enum)]
        shell: Shell,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli() {
        Cli::command().debug_assert();
    }
}
