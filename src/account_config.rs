use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use tracing::{debug, info};

pub fn read_accounts_json(
    accounts_config_path: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    debug!(
        "Reading account configuration from: {}",
        accounts_config_path
    );
    let file = File::open(accounts_config_path)?;
    let reader = BufReader::new(file);
    let accounts: HashMap<String, String> = serde_json::from_reader(reader)?;

    info!(
        "Loaded {} account mappings from configuration",
        accounts.len()
    );
    Ok(accounts)
}
