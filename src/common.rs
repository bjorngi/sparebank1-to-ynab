use chrono::{DateTime, Utc};

pub struct CommonTransaction {
    pub id: String,
    pub amount: i32,
    pub bank_account_key: String,
    pub date: DateTime<Utc>,
    pub memo: String,
    pub payee: String,
}
