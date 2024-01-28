use reqwest;
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
    description: String,
    #[serde(rename = "cleanedDescription")]
    cleaned_description: String,
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
        description: transaction.description.clone(),
        payee: transaction.cleaned_description.clone(),
        amount: transaction.amount,
        date: transaction_date,
    }

}

pub async fn get_transactions(access_token: String, accounts: Vec<String>) -> Result<Vec<Transaction>, reqwest::Error> {
    let url: &str = &format!("{}/transactions", BASE_API_URL);
    let params: Vec<(&str, &str)> = accounts.iter().map(|account| ("accountKey", account.as_str())).collect();

    let client = reqwest::Client::new();
    let transaction_respose: TransactionsResponse = client.get(url)
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

    let transactions: Vec<Transaction> = transaction_respose.transactions.iter().map(parse_transaction).collect();

    Ok(transactions)
}
