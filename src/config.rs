#[derive(Debug)]
pub struct Config {
    pub sparebank1_client_id: String,
    pub sparebank1_client_secret: String,
    pub ynab_access_token: String,
    pub ynab_budget_id: String,
    pub account_config_path: String,
    pub refresh_token_file_path: String,
    pub initial_refresh_token: String,
}

impl Config {
    pub fn new() -> Result<Self, dotenvy::Error> {
        dotenvy::dotenv()?;
        if let (
            Ok(sparebank1_client_id),
            Ok(sparebank1_client_secret),
            Ok(ynab_access_token),
            Ok(ynab_budget_id),
            Ok(account_config_path),
            Ok(refresh_token_file_path),
            Ok(initial_refresh_token),
        ) = (
            dotenvy::var("SPAREBANK1_CLIENT_ID"),
            dotenvy::var("SPAREBANK1_CLIENT_SECRET"),
            dotenvy::var("YNAB_ACCESS_TOKEN"),
            dotenvy::var("YNAB_BUDGET_ID"),
            dotenvy::var("ACCOUNT_CONFIG_PATH"),
            dotenvy::var("REFRESH_TOKEN_FILE_PATH"),
            dotenvy::var("INITIAL_REFRESH_TOKEN"),
        ) {
            Ok(Self {
                sparebank1_client_id,
                sparebank1_client_secret,
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
