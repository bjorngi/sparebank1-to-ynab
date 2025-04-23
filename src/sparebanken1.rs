use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::error::Error;

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

#[derive(Debug, Deserialize)]
struct AccountsResponse {
    accounts: Vec<Account>,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub name: String,
    pub balance: f32,
    pub key: String,
    #[serde(rename = "accountNumber")]
    pub account_number: String,
}

#[derive(Debug, Clone)]
pub struct Sparebanken1Client {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
}

impl Sparebanken1Client {
    /// Create a new Sparebanken1Client with the provided access token
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    /// Create a new client by refreshing the token
    pub async fn from_refresh_token(
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<(Self, String), Box<dyn Error>> {
        let (access_token, new_refresh_token) =
            Self::refresh_access_token(client_id, client_secret, refresh_token).await?;
        Ok((Self::new(access_token), new_refresh_token))
    }

    /// Update client with a new access token
    pub fn with_access_token(&mut self, access_token: String) {
        self.access_token = access_token;
    }

    /// Get current access token
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Refresh access token using the refresh token flow
    pub async fn refresh_access_token(
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<(String, String), Box<dyn Error>> {
        let client = reqwest::Client::new();
        let url = "https://api-auth.sparebank1.no/oauth/token";

        let body = format!(
            "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
            refresh_token, client_id, client_secret
        );

        let response: AuthResponse = client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?
            .json::<AuthResponse>()
            .await?;

        // Return both the access token and refresh token
        Ok((response.access_token, response.refresh_token))
    }

    /// Parse a transaction response into a Transaction struct
    fn parse_transaction(&self, transaction: &TransactionResponse) -> Transaction {
        let transaction_date = DateTime::from_timestamp(transaction.date / 1000, 0).unwrap();

        Transaction {
            id: transaction.id.clone(),
            account: transaction.account_key.clone(),
            description: transaction.description.clone().unwrap_or_default(),
            payee: transaction.cleaned_description.clone().unwrap_or_default(),
            amount: transaction.amount,
            date: transaction_date,
        }
    }

    /// Get transactions for the specified accounts
    pub async fn get_transactions(
        &self,
        accounts: Vec<String>,
    ) -> Result<Vec<Transaction>, reqwest::Error> {
        let url = format!("{}/transactions", BASE_API_URL);
        let params: Vec<(&str, &str)> = accounts
            .iter()
            .map(|account| ("accountKey", account.as_str()))
            .collect();

        let client = reqwest::Client::new();
        let transaction_response: TransactionsResponse = client
            .get(&url)
            .header("Authorization", &format!("Bearer {}", self.access_token))
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

        let transactions: Vec<Transaction> = transaction_response
            .transactions
            .iter()
            .map(|txn| self.parse_transaction(txn))
            .collect();

        Ok(transactions)
    }

    /// Get accounts for the authenticated user
    pub async fn get_accounts(&self) -> Result<Vec<Account>, reqwest::Error> {
        let url = format!("{}/accounts?includeCreditCardAccounts=true", BASE_API_URL);

        let accounts_response = reqwest::Client::new()
            .get(&url)
            .header("Authorization", &format!("Bearer {}", self.access_token))
            .header("accept", "application/vnd.sparebank1.v1+json")
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {
                eprintln!("Request error: {}", e);
                e
            })?
            .text()
            .await?;

        let accounts_json: AccountsResponse =
            serde_json::from_str(&accounts_response).expect("Failed to parse accounts response");

        Ok(accounts_json.accounts)
    }
}

// Legacy functions for backward compatibility
pub async fn get_transactions(
    access_token: &String,
    accounts: Vec<String>,
) -> Result<Vec<Transaction>, reqwest::Error> {
    let client = Sparebanken1Client::new(access_token.clone());
    client.get_transactions(accounts).await
}

pub async fn get_accounts(access_token: &String) -> Result<Vec<Account>, reqwest::Error> {
    let client = Sparebanken1Client::new(access_token.clone());
    client.get_accounts().await
}
