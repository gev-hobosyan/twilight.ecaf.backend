use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct APIError {
    message: String
}

impl APIError {
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}

impl From<&str> for APIError {
    fn from(value: &str) -> Self {
        APIError::new(value.into())
    }
}