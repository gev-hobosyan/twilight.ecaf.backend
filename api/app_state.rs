use std::env;
use sea_orm::{Database, DatabaseConnection};
use teloxide::Bot;
use teloxide::prelude::{AutoSend, ChatId, RequesterExt};

pub struct AppState {
    pub db: DatabaseConnection,
    pub bot: AutoSend<Bot>,
    pub chat_id: ChatId
}

impl AppState {
    pub async fn new() -> Result<Self, vercel_runtime::Error> {
        let db_url = env::var("NEON_DATABASE_URL")?;
        let db = Database::connect(db_url).await?;
        
        let chat_id = ChatId(env::var("ADMIN_ID")?.parse()?);

        let bot = Bot::new(env::var("BOT_TOKEN")?).auto_send();

        Ok(
            Self {
                db,
                bot,
                chat_id
            }
        )
    }
}