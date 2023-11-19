use chrono::{DateTime, Utc};
use rkyv::{Archive, Deserialize as RDeserialize, Serialize as RSerialize};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, Archive, RSerialize, RDeserialize, FromRow)]
pub struct SavedSession<'a> {
    pub id: i32,
    pub events: &'a [u8],
    pub session_data: &'a [u8],
    pub participants: &'a [u8],
    pub session_history: &'a [u8],
    pub final_classifications: &'a [u8],
    pub championship_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
