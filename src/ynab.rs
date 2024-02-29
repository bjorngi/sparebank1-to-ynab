use crate::sparebanken1;
use chrono_tz::Europe::Oslo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BASE_API_URL: &str = "https://api.ynab.com/v1";

#[derive(Debug, Serialize)]
struct CreateYnabTransactionRequest {
    transactions: Vec<CreateYnabTransaction>,
}

#[derive(Debug, Serialize)]
struct CreateYnabTransaction {
    date: String,
    account_id: String,
    amount: i64,
    payee_name: String,
    cleared: String,
    memo: String,
    import_id: String,
}

#[derive(Debug, Deserialize)]
struct CreateYnabTransactionResponse {
    data: CreateYnabTransactionResponseData,
}

#[derive(Debug, Deserialize)]
pub struct CreateYnabTransactionResponseData {
    pub transaction_ids: Vec<String>,
    pub duplicate_import_ids: Vec<String>,
}

fn parse_transactions(
    transactions: Vec<sparebanken1::Transaction>,
    account_config: &HashMap<String, String>,
) -> Vec<CreateYnabTransaction> {
    transactions
        .iter()
        .scan(Vec::new(), |state, t| {
            let oslo_time = t.date.with_timezone(&Oslo);
            let formated_date = oslo_time.format("%Y-%m-%d").to_string();
            let account_id = account_config.get(&t.account).unwrap();

            // Check if same transactions has been imported before
            let import_prefix = format!("SB1:{}:{}", t.amount, formated_date);
            state.push(import_prefix.clone());
            let import_count = state
                .iter()
                .filter(|id| id.starts_with(&import_prefix))
                .count();
            let import_id = format!("{}:{}", import_prefix, import_count);

            Some(CreateYnabTransaction {
                date: formated_date,
                account_id: account_id.clone(),
                amount: (t.amount * 1000.0) as i64,
                payee_name: t.payee.clone(),
                cleared: String::from("cleared"),
                memo: t.description.clone(),
                import_id,
            })
        })
        .collect()
}

pub async fn add_transactions(
    ynab_access_token: &str,
    ynab_budget_id: &str,
    account_config: HashMap<String, String>,
    transactions: Vec<sparebanken1::Transaction>,
) -> Result<CreateYnabTransactionResponseData, reqwest::Error> {
    let url: &str = &format!("{}/budgets/{}/transactions", BASE_API_URL, ynab_budget_id);
    let ynab_transactions: Vec<CreateYnabTransaction> =
        parse_transactions(transactions, &account_config);

    let data = CreateYnabTransactionRequest {
        transactions: ynab_transactions,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Authorization", &format!("Bearer {}", ynab_access_token))
        .json(&data)
        .send()
        .await?
        .json::<CreateYnabTransactionResponse>()
        .await?;

    Ok(response.data)
}

#[derive(Debug, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    closed: bool,
}
#[derive(Debug, Deserialize)]
struct YnabAccountResponse {
    accounts: Vec<Account>,
}

#[derive(Debug, Deserialize)]
struct YnabAccountsResponse {
    data: YnabAccountResponse,
}

pub async fn get_accounts(
    ynab_access_token: &str,
    ynab_budget_id: &str,
) -> Result<Vec<Account>, reqwest::Error> {
    let url = format!("{BASE_API_URL}/budgets/{}/accounts", ynab_budget_id);

    let response = reqwest::Client::new()
        .get(url)
        .header("Authorization", &format!("Bearer {}", ynab_access_token))
        .send()
        .await
        .expect("Failed to get account from YNAB")
        .json::<YnabAccountsResponse>()
        .await
        .expect("Could not parse data");

    let filteredAccounts = response
        .data
        .accounts
        .iter()
        .filter(|&acc| !acc.closed)
        .cloned()
        .collect();

    Ok(filteredAccounts)
}

#[derive(Debug, Deserialize)]
pub struct Budget {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct YnabBudgetsResponse {
    budgets: Vec<Budget>,
}

#[derive(Debug, Deserialize)]
struct YnabBudgetsDataResponse {
    data: YnabBudgetsResponse,
}
pub async fn get_budgets(ynab_access_token: &str) -> Result<Vec<Budget>, reqwest::Error> {
    let url = format!("{BASE_API_URL}/budgets/");
    let response = reqwest::Client::new()
        .get(url)
        .header("Authorization", &format!("Bearer {}", ynab_access_token))
        .send()
        .await?
        .json::<YnabBudgetsDataResponse>()
        .await?;

    Ok(response.data.budgets)
}
