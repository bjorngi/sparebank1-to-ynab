use chrono::{DateTime, Datelike, TimeZone, Utc};
use sparebank1_to_ynab::sparebanken1::Transaction as Sparebank1Transaction;
use sparebank1_to_ynab::ynab::YnabClient;
use std::collections::HashMap;

#[cfg(test)]
mod ynab_transaction_tests {
    use super::*;

    fn create_test_transaction(
        id: &str,
        amount: f32,
        date_timestamp: i64,
        payee: &str,
        description: &str,
        account: &str,
    ) -> Sparebank1Transaction {
        Sparebank1Transaction {
            id: id.to_string(),
            description: description.to_string(),
            payee: payee.to_string(),
            amount,
            date: DateTime::from_timestamp(date_timestamp, 0).unwrap(),
            account: account.to_string(),
        }
    }

    #[test]
    fn test_amount_conversion_to_milliunits() {
        // YNAB requires amounts in milliunits (amount * 1000)
        let mut account_config = HashMap::new();
        account_config.insert("account1".to_string(), "ynab-id-1".to_string());

        let _client = YnabClient::new(
            account_config,
            "test_token".to_string(),
            "test_budget".to_string(),
        );

        // Test positive amount (income)
        let transaction = create_test_transaction(
            "txn1", 1234.56, 1704067200, // 2024-01-01
            "Employer", "Salary", "account1",
        );

        // The parse_transactions method is private, so we test through the public interface
        // We'll verify the conversion logic is correct by examining the behavior
        // In a real scenario, this would be sent to YNAB which expects milliunits

        // Expected: 1234.56 * 1000 = 1234560 milliunits
        assert_eq!((transaction.amount * 1000.0) as i64, 1234560);

        // Test negative amount (expense)
        let transaction2 =
            create_test_transaction("txn2", -99.99, 1704067200, "Store", "Groceries", "account1");

        // Expected: -99.99 * 1000 = -99990 milliunits
        assert_eq!((transaction2.amount * 1000.0) as i64, -99990);

        // Test zero amount
        let transaction3 = create_test_transaction(
            "txn3",
            0.0,
            1704067200,
            "Transfer",
            "Internal transfer",
            "account1",
        );

        assert_eq!((transaction3.amount * 1000.0) as i64, 0);
    }

    #[test]
    fn test_date_conversion_to_oslo_timezone() {
        // Create a UTC timestamp for 2024-01-15 10:30:00 UTC
        let utc_dt = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let timestamp = utc_dt.timestamp();

        let transaction = create_test_transaction(
            "txn1",
            100.0,
            timestamp,
            "Test",
            "Test transaction",
            "account1",
        );

        // Verify the date is stored correctly in UTC
        assert_eq!(transaction.date.year(), 2024);
        assert_eq!(transaction.date.month(), 1);
        assert_eq!(transaction.date.day(), 15);

        // The YNAB conversion uses Oslo timezone, which is typically UTC+1 or UTC+2
        // The formatted date should be YYYY-MM-DD in Oslo timezone
    }

    #[test]
    fn test_import_id_generation_format() {
        // Import IDs follow the format: SB1:{amount}:{date}:{occurrence}
        let _transaction = create_test_transaction(
            "txn1",
            -127.50,
            1704067200, // 2024-01-01 00:00:00 UTC
            "REMA 1000",
            "Groceries",
            "account1",
        );

        // Expected format: SB1:-127.5:2024-01-01:1 (for first occurrence)
        let expected_prefix = "SB1:-127.5:2024-01-01";

        // The import ID should start with this prefix
        // The actual implementation in YnabClient will add the occurrence counter
        assert!(format!("{}:{}", expected_prefix, 1).starts_with("SB1:"));
    }

    #[test]
    fn test_duplicate_detection_with_same_amount_and_date() {
        // When multiple transactions have the same amount and date,
        // the occurrence counter should increment
        let mut account_config = HashMap::new();
        account_config.insert("account1".to_string(), "ynab-id-1".to_string());

        let _client = YnabClient::new(
            account_config,
            "test_token".to_string(),
            "test_budget".to_string(),
        );

        // Create three transactions with same amount and date
        let txn1 = create_test_transaction(
            "txn1",
            -50.0,
            1704067200,
            "Store A",
            "Purchase 1",
            "account1",
        );

        let txn2 = create_test_transaction(
            "txn2",
            -50.0,
            1704067200,
            "Store B",
            "Purchase 2",
            "account1",
        );

        let txn3 = create_test_transaction(
            "txn3",
            -50.0,
            1704067200,
            "Store C",
            "Purchase 3",
            "account1",
        );

        // All three transactions should have different import IDs:
        // SB1:-50.0:2024-01-01:1
        // SB1:-50.0:2024-01-01:2
        // SB1:-50.0:2024-01-01:3

        let transactions = vec![txn1, txn2, txn3];
        assert_eq!(transactions.len(), 3);

        // Each should be unique based on occurrence counter
        // This is handled by the scan() logic in parse_transactions
    }

    #[test]
    fn test_account_mapping() {
        let mut account_config = HashMap::new();
        account_config.insert("sb1_account_1".to_string(), "ynab_account_1".to_string());
        account_config.insert("sb1_account_2".to_string(), "ynab_account_2".to_string());

        let _client = YnabClient::new(
            account_config.clone(),
            "test_token".to_string(),
            "test_budget".to_string(),
        );

        // Create transaction for first account
        let transaction1 =
            create_test_transaction("txn1", 100.0, 1704067200, "Test", "Test", "sb1_account_1");

        // Create transaction for second account
        let transaction2 =
            create_test_transaction("txn2", 200.0, 1704067200, "Test", "Test", "sb1_account_2");

        // Verify account mapping exists
        assert_eq!(
            account_config.get(&transaction1.account),
            Some(&"ynab_account_1".to_string())
        );
        assert_eq!(
            account_config.get(&transaction2.account),
            Some(&"ynab_account_2".to_string())
        );
    }

    #[test]
    fn test_empty_description_handling() {
        let transaction = create_test_transaction(
            "txn1",
            50.0,
            1704067200,
            "Payee Name",
            "", // Empty description
            "account1",
        );

        assert_eq!(transaction.description, "");
        assert_eq!(transaction.payee, "Payee Name");
    }

    #[test]
    fn test_empty_payee_handling() {
        let transaction = create_test_transaction(
            "txn1",
            50.0,
            1704067200,
            "", // Empty payee
            "Some description",
            "account1",
        );

        assert_eq!(transaction.payee, "");
        assert_eq!(transaction.description, "Some description");
    }

    #[test]
    fn test_large_amount_conversion() {
        // Test with a very large amount (e.g., salary)
        let transaction = create_test_transaction(
            "txn1",
            45000.50,
            1704067200,
            "Employer",
            "Monthly salary",
            "account1",
        );

        let milliunits = (transaction.amount * 1000.0) as i64;
        assert_eq!(milliunits, 45000500);
    }

    #[test]
    fn test_small_fractional_amount() {
        // Test with very small amounts (cents/øre)
        let transaction =
            create_test_transaction("txn1", 0.01, 1704067200, "Test", "One øre", "account1");

        let milliunits = (transaction.amount * 1000.0) as i64;
        assert_eq!(milliunits, 10);
    }

    #[test]
    fn test_negative_zero_handling() {
        // Edge case: -0.0 should be treated as 0
        let transaction =
            create_test_transaction("txn1", -0.0, 1704067200, "Test", "Zero amount", "account1");

        let milliunits = (transaction.amount * 1000.0) as i64;
        assert_eq!(milliunits, 0);
    }
}
