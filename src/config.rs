use std::env;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// ConfigError represents all possible errors when initializing configuration
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Failed to load .env file: {0}")]
    DotEnvError(#[from] dotenvy::Error),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

/// Config holds the application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub sparebank1_client_id: String,
    pub sparebank1_client_secret: String,
    pub sparebank1_fin_inst: String,
    pub ynab_access_token: String,
    pub ynab_budget_id: String,
    pub account_config_path: String,
    pub refresh_token_file_path: String,
    pub initial_refresh_token: String,
    pub dry_run: bool,
}

impl Config {
    /// Creates a new Config from environment variables
    pub fn new() -> Result<Self, ConfigError> {
        // Load environment variables from .env file if present
        if let Err(e) = dotenvy::dotenv() {
            warn!(
                "No .env file found, using system environment variables: {}",
                e
            );
        } else {
            debug!("Loaded configuration from .env file");
        }

        let config = Self {
            sparebank1_client_id: Self::get_env_or_error("SPAREBANK1_CLIENT_ID")?,
            sparebank1_client_secret: Self::get_env_or_error("SPAREBANK1_CLIENT_SECRET")?,
            sparebank1_fin_inst: Self::get_env_or_error("SPAREBANK1_FIN_INST")?,
            ynab_access_token: Self::get_env_or_error("YNAB_ACCESS_TOKEN")?,
            ynab_budget_id: Self::get_env_or_error("YNAB_BUDGET_ID")?,
            account_config_path: Self::get_env_or_error("ACCOUNT_CONFIG_PATH")?,
            refresh_token_file_path: Self::get_env_with_default(
                "REFRESH_TOKEN_FILE_PATH",
                "refresh_token.txt",
            )?,
            initial_refresh_token: Self::get_env_or_error("INITIAL_REFRESH_TOKEN")?,
            dry_run: Self::get_env_bool("DRY_RUN"),
        };

        // Validate the configuration
        config.validate()?;

        info!("Configuration loaded successfully");
        debug!("Budget ID: {}", config.ynab_budget_id);
        debug!("Account config path: {}", config.account_config_path);

        Ok(config)
    }

    /// Creates a new Config with explicitly provided values (useful for testing and setup)
    pub fn with_values(
        sparebank1_client_id: String,
        sparebank1_client_secret: String,
        sparebank1_fin_inst: String,
        ynab_access_token: String,
        ynab_budget_id: String,
        account_config_path: String,
        refresh_token_file_path: Option<String>,
        initial_refresh_token: String,
    ) -> Result<Self, ConfigError> {
        let config = Self {
            sparebank1_client_id,
            sparebank1_client_secret,
            sparebank1_fin_inst,
            ynab_access_token,
            ynab_budget_id,
            account_config_path,
            refresh_token_file_path: refresh_token_file_path
                .unwrap_or_else(|| "refresh_token.txt".to_string()),
            initial_refresh_token,
            dry_run: false,
        };

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Get an environment variable or return an error if it's not present
    fn get_env_or_error(name: &str) -> Result<String, ConfigError> {
        env::var(name).map_err(|e| ConfigError::EnvVarError(e))
    }

    /// Get an environment variable with a default value if not present
    fn get_env_with_default(name: &str, default: &str) -> Result<String, ConfigError> {
        match env::var(name) {
            Ok(val) => Ok(val),
            Err(std::env::VarError::NotPresent) => Ok(default.to_string()),
            Err(e) => Err(ConfigError::EnvVarError(e)),
        }
    }

    /// Get a boolean environment variable (true if set to "1", "true", "yes", case-insensitive)
    fn get_env_bool(name: &str) -> bool {
        match env::var(name) {
            Ok(val) => {
                let val_lower = val.to_lowercase();
                val_lower == "1" || val_lower == "true" || val_lower == "yes"
            }
            Err(_) => false,
        }
    }

    /// Validates the configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Check that required IDs and tokens are not empty
        if self.sparebank1_client_id.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "SPAREBANK1_CLIENT_ID cannot be empty".to_string(),
            ));
        }
        if self.sparebank1_client_secret.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "SPAREBANK1_CLIENT_SECRET cannot be empty".to_string(),
            ));
        }
        if self.ynab_access_token.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "YNAB_ACCESS_TOKEN cannot be empty".to_string(),
            ));
        }
        if self.ynab_budget_id.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "YNAB_BUDGET_ID cannot be empty".to_string(),
            ));
        }

        // Check that paths exist or are in expected locations
        if !PathBuf::from(&self.account_config_path).exists() {
            warn!(
                "Account config file does not exist at {}",
                self.account_config_path
            );
        }

        Ok(())
    }
}
