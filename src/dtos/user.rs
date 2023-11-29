use garde::Validate;
use serde::Deserialize;
use serde_trim::{option_string_trim, string_trim};

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[serde(default, deserialize_with = "option_string_trim")]
    #[garde(inner(ascii, length(min = 3, max = 20)))]
    pub username: Option<String>,
    #[serde(default, deserialize_with = "option_string_trim")]
    #[garde(inner(ascii, length(min = 10, max = 100)))]
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddUser {
    #[serde(deserialize_with = "string_trim")]
    #[garde(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RemoveUser {
    #[serde(deserialize_with = "string_trim")]
    #[garde(email)]
    pub email: String,
}
