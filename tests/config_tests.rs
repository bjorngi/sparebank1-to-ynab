use sparebank1_to_ynab::config::Config;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_with_values() {
        // Test creating a config with explicit values
        let result = Config::with_values(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            Some("refresh_token.txt".to_string()),
            "test_refresh_token".to_string(),
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.sparebank1_client_id, "test_client_id");
        assert_eq!(config.sparebank1_client_secret, "test_client_secret");
        assert_eq!(config.sparebank1_fin_inst, "test_fin_inst");
        assert_eq!(config.ynab_access_token, "test_ynab_token");
        assert_eq!(config.ynab_budget_id, "test_budget_id");
        assert_eq!(config.account_config_path, "/tmp/accounts.json");
        assert_eq!(config.refresh_token_file_path, "refresh_token.txt");
        assert_eq!(config.initial_refresh_token, "test_refresh_token");
        assert_eq!(config.dry_run, false);
    }

    #[test]
    fn test_config_validation_empty_client_id() {
        let result = Config::with_values(
            "".to_string(), // Empty client ID
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            None,
            "test_refresh_token".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CLIENT_ID"));
    }

    #[test]
    fn test_config_validation_empty_client_secret() {
        let result = Config::with_values(
            "test_client_id".to_string(),
            "".to_string(), // Empty client secret
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            None,
            "test_refresh_token".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CLIENT_SECRET"));
    }

    #[test]
    fn test_config_validation_empty_ynab_token() {
        let result = Config::with_values(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "".to_string(), // Empty YNAB token
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            None,
            "test_refresh_token".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ACCESS_TOKEN"));
    }

    #[test]
    fn test_config_validation_empty_budget_id() {
        let result = Config::with_values(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "".to_string(), // Empty budget ID
            "/tmp/accounts.json".to_string(),
            None,
            "test_refresh_token".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("BUDGET_ID"));
    }

    #[test]
    fn test_config_validation_whitespace_only() {
        let result = Config::with_values(
            "   ".to_string(), // Whitespace only
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            None,
            "test_refresh_token".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_file_path_default() {
        let result = Config::with_values(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            None, // No refresh token file path provided
            "test_refresh_token".to_string(),
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.refresh_token_file_path, "refresh_token.txt");
    }

    #[test]
    fn test_refresh_token_file_path_custom() {
        let result = Config::with_values(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_fin_inst".to_string(),
            "test_ynab_token".to_string(),
            "test_budget_id".to_string(),
            "/tmp/accounts.json".to_string(),
            Some("/custom/path/token.txt".to_string()),
            "test_refresh_token".to_string(),
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.refresh_token_file_path, "/custom/path/token.txt");
    }
}
