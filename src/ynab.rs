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

pub struct YnabClient {
    ynab_token: String,
    ynab_budget: String,
    account_config: HashMap<String, String>,
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

impl YnabClient {
    pub fn new(
        account_config: HashMap<String, String>,
        ynab_token: String,
        ynab_budget: String,
    ) -> Self {
        YnabClient {
            ynab_token,
            ynab_budget,
            account_config,
        }
    }

    fn parse_transactions(
        &self,
        transactions: &[sparebanken1::Transaction],
    ) -> Vec<CreateYnabTransaction> {
        transactions
            .iter()
            .scan(Vec::new(), |state, t| {
                let oslo_time = t.date.with_timezone(&Oslo);
                let formated_date = oslo_time.format("%Y-%m-%d").to_string();
                let account_id = self.account_config.get(&t.account).unwrap();

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
        &self,
        transactions: Vec<sparebanken1::Transaction>,
    ) -> Result<CreateYnabTransactionResponseData, reqwest::Error> {
        let url = format!("{}/budgets/{}/transactions", BASE_API_URL, self.ynab_budget);
        let ynab_transactions = self.parse_transactions(&transactions);

        let data = CreateYnabTransactionRequest {
            transactions: ynab_transactions,
        };

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", &format!("Bearer {}", self.ynab_token))
            .json(&data)
            .send()
            .await?
            .json::<CreateYnabTransactionResponse>()
            .await?;

        Ok(response.data)
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>, reqwest::Error> {
        let url = format!("{BASE_API_URL}/budgets/{}/accounts", self.ynab_budget);

        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", &format!("Bearer {}", self.ynab_token))
            .send()
            .await
            .expect("Failed to get account from YNAB")
            .json::<YnabAccountsResponse>()
            .await
            .expect("Could not parse data");

        let filtered_accounts = response
            .data
            .accounts
            .iter()
            .filter(|&acc| !acc.closed)
            .cloned()
            .collect();

        Ok(filtered_accounts)
    }

    pub async fn get_budgets(&self) -> Result<Vec<Budget>, reqwest::Error> {
        let url = format!("{BASE_API_URL}/budgets/");
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", &format!("Bearer {}", self.ynab_token))
            .send()
            .await?
            .json::<YnabBudgetsDataResponse>()
            .await?;

        Ok(response.data.budgets)
    }
}