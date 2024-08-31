use std::net::IpAddr;

use academy_testing::recaptcha;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Recaptcha { host, port, secret } => {
            recaptcha::start_server(host, port, secret).await?
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
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the recaptcha testing server
    Recaptcha {
        #[arg(long, default_value = "127.0.0.1")]
        host: IpAddr,
        #[arg(long, default_value = "8001")]
        port: u16,
        #[arg(long, default_value = "test-secret")]
        secret: String,
    },
    /// Generate shell completions
    Completion {
        /// The shell to generate completions for
        #[clap(value_enum)]
        shell: Shell,
    },
}
