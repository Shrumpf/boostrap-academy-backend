use academy_config::Config;
use clap::Subcommand;
use user::AdminUserCommand;

mod user;

#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Manager user accounts
    #[command(aliases(["u"]))]
    User {
        #[command(subcommand)]
        command: AdminUserCommand,
    },
}

impl AdminCommand {
    pub async fn invoke(self, config: Config) -> anyhow::Result<()> {
        match self {
            AdminCommand::User { command } => command.invoke(config).await,
        }
    }
}
