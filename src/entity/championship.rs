use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Championship {
    pub id: i32,
    pub port: i16,
    pub name: String,
    pub user_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
