use chrono::{DateTime, Utc};

pub struct CommonTransaction {
    id: String,
    amount: i32,
    bankAccountKey: String,
    date: DateTime<Utc>,
    memo: String,
    payee: String,
}
