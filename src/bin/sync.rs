use sparebank1_to_ynab::account_config;
use sparebank1_to_ynab::auth_data;
use sparebank1_to_ynab::config::Config;
use sparebank1_to_ynab::sparebanken1;
use sparebank1_to_ynab::ynab::YnabClient;
use std::error::Error;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting SpareBank1 to YNAB sync");

    let config = Config::new()?;
    info!("Fetching access token");
    let access_token = match auth_data::get_access_token(&config).await {
        Ok(token) => {
            info!("Successfully obtained access token");
            token
        }
        Err(e) => {
            error!("Failed to get access token: {}", e);
            return Err(e);
        }
    };

    info!(
        "Loading account configuration from {}",
        config.account_config_path
    );
    let account_config = account_config::read_accounts_json(&config.account_config_path)?;
    let accounts: Vec<String> = account_config.keys().cloned().collect();
    info!("Configured accounts: {}", accounts.len());

    info!("Fetching transactions from SpareBank1");
    let transactions = sparebanken1::get_transactions(&access_token, accounts).await?;
    info!("Retrieved {} transactions", transactions.len());

    // Create YnabClient instance
    let ynab_client = YnabClient::new(
        account_config,
        config.ynab_access_token.clone(),
        config.ynab_budget_id.clone(),
    );

    // Use YnabClient to add transactions
    info!("Importing transactions to YNAB");
    let ynab_response = ynab_client.add_transactions(transactions).await?;

    let now = chrono::offset::Local::now();
    info!("Sync completed at {}", now);
    info!(
        "Added {} new transactions",
        ynab_response.transaction_ids.len()
    );
    info!(
        "Skipped {} duplicate transactions",
        ynab_response.duplicate_import_ids.len()
    );

    Ok(())
}
