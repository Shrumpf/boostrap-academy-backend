use academy_config::Config;
use academy_core_user_contracts::user::{UserCreateCommand, UserService};
use academy_di::Provides;
use academy_persistence_contracts::{Database as _, Transaction};
use clap::Subcommand;

use crate::{
    cache, database, email,
    environment::{
        types::{self, Database},
        ConfigProvider, Provider,
    },
};

#[derive(Debug, Subcommand)]
pub enum AdminUserCommand {
    /// Create a new user account
    #[command(aliases(["c", "new", "n", "+"]))]
    Create {
        #[arg(long)]
        admin: bool,
        #[arg(long)]
        disabled: bool,
        #[arg(long)]
        verified: bool,
        name: String,
        email: String,
        password: String,
    },
}

impl AdminUserCommand {
    pub async fn invoke(self, config: Config) -> anyhow::Result<()> {
        match self {
            AdminUserCommand::Create {
                admin,
                name,
                email,
                password,
                disabled,
                verified,
            } => create(config, name, email, password, admin, !disabled, verified).await,
        }
    }
}

async fn create(
    config: Config,
    name: String,
    email: String,
    password: String,
    admin: bool,
    enabled: bool,
    email_verified: bool,
) -> anyhow::Result<()> {
    let database = database::connect(&config.database).await?;
    let cache = cache::connect(&config.cache).await?;
    let email_service = email::connect(&config.email).await?;
    let config_provider = ConfigProvider::new(&config)?;
    let mut provider = Provider::new(config_provider, database, cache, email_service);

    let db: Database = provider.provide();
    let mut txn = db.begin_transaction().await?;

    let user_service: types::User = provider.provide();
    let user = user_service
        .create(
            &mut txn,
            UserCreateCommand {
                name: name.clone().try_into()?,
                display_name: name.try_into()?,
                email: email.parse()?,
                password: Some(password.try_into()?),
                admin,
                enabled,
                email_verified,
                oauth2_registration: None,
            },
        )
        .await?;

    txn.commit().await?;

    println!("User has been created:");
    println!("{user:#?}");

    Ok(())
}
