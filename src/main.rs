mod config;
mod sparebanken1;
mod ynab;
mod auth_data;
mod account_config;

use config::Config;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;
    let access_token =  auth_data::get_access_token(&config).await?;

    let account_config = account_config::read_accounts_json(&config.account_config_path)?;
    let accounts = account_config.keys().cloned().collect();

    let transactions = sparebanken1::get_transactions(access_token, accounts).await?;
    let ynab_response = ynab::add_transactions(&config, account_config, transactions).await?;

    let now = chrono::offset::Local::now();
    println!("--- {} ---", now);
    println!("{} Added transactions", ynab_response.transaction_ids.len());
    println!("{} Duplicate transactions", ynab_response.duplicate_import_ids.len());

    Ok(())
}
