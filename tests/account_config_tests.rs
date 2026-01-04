use sparebank1_to_ynab::account_config;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[cfg(test)]
mod account_config_tests {
    use super::*;

    #[test]
    fn test_read_valid_accounts_json() {
        let temp_file = "/tmp/test_accounts_valid.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = r#"{
            "account_key_1": "ynab_id_1",
            "account_key_2": "ynab_id_2",
            "account_key_3": "ynab_id_3"
        }"#;
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 3);
        assert_eq!(
            accounts.get("account_key_1"),
            Some(&"ynab_id_1".to_string())
        );
        assert_eq!(
            accounts.get("account_key_2"),
            Some(&"ynab_id_2".to_string())
        );
        assert_eq!(
            accounts.get("account_key_3"),
            Some(&"ynab_id_3".to_string())
        );

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_empty_accounts_json() {
        let temp_file = "/tmp/test_accounts_empty.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = "{}";
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 0);

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = account_config::read_accounts_json("/nonexistent/path/accounts.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_invalid_json() {
        let temp_file = "/tmp/test_accounts_invalid.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = "{ invalid json content !!";
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_err());

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_single_account() {
        let temp_file = "/tmp/test_accounts_single.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = r#"{
            "single_account": "single_ynab_id"
        }"#;
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(
            accounts.get("single_account"),
            Some(&"single_ynab_id".to_string())
        );

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_accounts_with_special_characters() {
        let temp_file = "/tmp/test_accounts_special.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = r#"{
            "account-with-dash": "ynab-id-1",
            "account_with_underscore": "ynab_id_2",
            "account.with.dots": "ynab.id.3"
        }"#;
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 3);
        assert_eq!(
            accounts.get("account-with-dash"),
            Some(&"ynab-id-1".to_string())
        );
        assert_eq!(
            accounts.get("account_with_underscore"),
            Some(&"ynab_id_2".to_string())
        );
        assert_eq!(
            accounts.get("account.with.dots"),
            Some(&"ynab.id.3".to_string())
        );

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_accounts_with_unicode() {
        let temp_file = "/tmp/test_accounts_unicode.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = r#"{
            "norwegian_account_æøå": "ynab_id_1",
            "emoji_account_🏦": "ynab_id_2"
        }"#;
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains_key("norwegian_account_æøå"));
        assert!(accounts.contains_key("emoji_account_🏦"));

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_accounts_preserves_all_mappings() {
        let temp_file = "/tmp/test_accounts_many.json";
        let mut file = fs::File::create(temp_file).unwrap();

        let mut expected: HashMap<String, String> = HashMap::new();
        for i in 1..=10 {
            expected.insert(format!("account_{}", i), format!("ynab_id_{}", i));
        }

        let json_content = serde_json::to_string_pretty(&expected).unwrap();
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 10);

        for i in 1..=10 {
            assert_eq!(
                accounts.get(&format!("account_{}", i)),
                Some(&format!("ynab_id_{}", i))
            );
        }

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_accounts_wrong_structure() {
        let temp_file = "/tmp/test_accounts_array.json";
        let mut file = fs::File::create(temp_file).unwrap();
        // Array instead of object
        let json_content = r#"["account1", "account2"]"#;
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        assert!(result.is_err());

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_read_accounts_null_values() {
        let temp_file = "/tmp/test_accounts_null.json";
        let mut file = fs::File::create(temp_file).unwrap();
        let json_content = r#"{
            "account1": "ynab_id_1",
            "account2": null
        }"#;
        file.write_all(json_content.as_bytes()).unwrap();

        let result = account_config::read_accounts_json(temp_file);
        // This should fail because values should be strings, not null
        assert!(result.is_err());

        fs::remove_file(temp_file).ok();
    }
}
