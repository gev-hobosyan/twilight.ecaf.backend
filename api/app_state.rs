use sea_orm::{Database, DatabaseConnection};

pub struct AppState {
    pub db: DatabaseConnection
}

impl AppState {
    pub async fn new(database_url: String) -> Self {
        let db = Database::connect(database_url).await.unwrap();

        Self {
            db
        }
    }
}