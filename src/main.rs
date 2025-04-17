mod account_config;
mod auth_data;
mod common;
mod config;
mod sparebanken1;
mod ynab;

use config::Config;
use std::error::Error;
use ynab::YnabClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;
    let access_token = auth_data::get_access_token(&config)
        .await
        .expect("Couldt not get access_token");

    let account_config = account_config::read_accounts_json(&config.account_config_path)?;
    let accounts = account_config.keys().cloned().collect();

    let transactions = sparebanken1::get_transactions(&access_token, accounts).await?;
    
    // Create YnabClient instance
    let ynab_client = YnabClient::new(
        account_config,
        config.ynab_access_token.clone(),
        config.ynab_budget_id.clone(),
    );
    
    // Use YnabClient to add transactions
    let ynab_response = ynab_client.add_transactions(transactions).await?;

    let now = chrono::offset::Local::now();
    println!("--- {} ---", now);
    println!("{} Added transactions", ynab_response.transaction_ids.len());
    println!(
        "{} Duplicate transactions",
        ynab_response.duplicate_import_ids.len()
    );

    Ok(())
}
