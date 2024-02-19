use chrono::{DateTime, Utc};
use serde::Deserialize;

const BASE_API_URL: &str = "https://api.sparebank1.no/personal/banking";

#[derive(Debug, Deserialize)]
struct TransactionsResponse {
    transactions: Vec<TransactionResponse>,
}
#[derive(Debug, Deserialize)]
struct TransactionResponse {
    id: String,
    amount: f32,
    description: Option<String>,
    #[serde(rename = "cleanedDescription")]
    cleaned_description: Option<String>,
    #[serde(rename = "accountKey")]
    account_key: String,
    date: i64,
}

#[derive(Debug)]
pub struct Transaction {
    pub id: String,
    pub description: String,
    pub payee: String,
    pub amount: f32,
    pub date: DateTime<Utc>,
    pub account: String,
}

fn parse_transaction(transaction: &TransactionResponse) -> Transaction {
    let transaction_date = DateTime::from_timestamp(transaction.date / 1000, 0).unwrap();

    Transaction {
        id: transaction.id.clone(),
        account: transaction.account_key.clone(),
        description: transaction.description.clone().unwrap_or("".to_string()),
        payee: transaction
            .cleaned_description
            .clone()
            .unwrap_or("".to_string()),
        amount: transaction.amount,
        date: transaction_date,
    }
}

pub async fn get_transactions(
    access_token: &String,
    accounts: Vec<String>,
) -> Result<Vec<Transaction>, reqwest::Error> {
    let url: &str = &format!("{}/transactions", BASE_API_URL);
    let params: Vec<(&str, &str)> = accounts
        .iter()
        .map(|account| ("accountKey", account.as_str()))
        .collect();

    let client = reqwest::Client::new();
    let transaction_respose: TransactionsResponse = client
        .get(url)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.sparebank1.v1+json")
        .query(&params)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            eprintln!("Request error: {}", e);
            e
        })?
        .json::<TransactionsResponse>()
        .await?;

    let transactions: Vec<Transaction> = transaction_respose
        .transactions
        .iter()
        .map(parse_transaction)
        .collect();

    Ok(transactions)
}

#[derive(Debug, Deserialize)]
struct AccountMetadataResponse {
    accountKey: String,
}

#[derive(Debug, Deserialize)]
struct AccountResponse {
    #[serde(rename = "accountName")]
    account_name: String,
    #[serde(rename = "accountNumber")]
    account_number: String,
    balance: String,
    metadata: AccountMetadataResponse,
}

#[derive(Debug, Deserialize)]
struct AccountsResponse {
    accounts: Vec<AccountResponse>,
}

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub balance: f32,
    pub key: String,
    pub account_number: String,
}

fn parse_account(account: &AccountResponse) -> Account {
    Account {
        name: account.account_name.clone(),
        balance: account.balance.parse::<f32>().unwrap(),
        key: account.metadata.accountKey.clone(),
        account_number: account.account_number.clone(),
    }
}

pub async fn get_accounts(access_token: &String) -> Result<Vec<Account>, reqwest::Error> {
    let url = format!("{}/accounts", BASE_API_URL);

    let accounts_response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            eprintln!("Request error: {}", e);
            e
        })?
        .json::<Vec<AccountResponse>>()
        .await?;

    let accounts: Vec<Account> = accounts_response.iter().map(parse_account).collect();

    Ok(accounts)
}
