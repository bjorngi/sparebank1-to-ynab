use std::env;

#[derive(Debug)]
pub struct Config {
    pub sparebank1_client_id: String,
    pub sparebank1_client_secret: String,
    pub sparebank1_fin_inst: String,
    pub ynab_access_token: String,
    pub ynab_budget_id: String,
    pub account_config_path: String,
    pub refresh_token_file_path: String,
    pub initial_refresh_token: String,
}

impl Config {
    pub fn new() -> Result<Self, dotenvy::Error> {
        if let Err(_) = dotenvy::dotenv() {
            println!("No .env file found, loading environment variables from system");
        }

        if let (
            Ok(sparebank1_client_id),
            Ok(sparebank1_client_secret),
            Ok(sparebank1_fin_inst),
            Ok(ynab_access_token),
            Ok(ynab_budget_id),
            Ok(account_config_path),
            Ok(refresh_token_file_path),
            Ok(initial_refresh_token),
        ) = (
            env::var("SPAREBANK1_CLIENT_ID"),
            env::var("SPAREBANK1_CLIENT_SECRET"),
            env::var("SPAREBANK1_FIN_INST"),
            env::var("YNAB_ACCESS_TOKEN"),
            env::var("YNAB_BUDGET_ID"),
            env::var("ACCOUNT_CONFIG_PATH"),
            env::var("REFRESH_TOKEN_FILE_PATH"),
            env::var("INITIAL_REFRESH_TOKEN"),
        ) {
            Ok(Self {
                sparebank1_client_id,
                sparebank1_client_secret,
                sparebank1_fin_inst,
                ynab_access_token,
                ynab_budget_id,
                account_config_path,
                refresh_token_file_path,
                initial_refresh_token,
            })
        } else {
            Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent))
        }
    }
}
