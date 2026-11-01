use std::env;
use std::sync::{Arc};
use tokio::sync::Mutex;
use sea_orm::{Database, DatabaseConnection};
use teloxide::Bot;
use teloxide::prelude::{ChatId};
use tokio::sync::broadcast;
use crate::entity::public::project;
use crate::vote_event::VoteEvent;

pub struct AppState {
    pub db: DatabaseConnection,
    pub bot: Bot,
    pub chat_id: ChatId,
    pub tx: broadcast::Sender<VoteEvent>,
    pub vote: Arc<Mutex<VoteState>>
}

impl AppState {
    pub async fn new() -> Result<Self, vercel_runtime::Error> {
        let (tx, _) = broadcast::channel(100);
        let db_url = env::var("NEON_DATABASE_URL")?;
        let db = Database::connect(db_url).await?;

        let chat_id = ChatId(env::var("ADMIN_ID")?.parse()?);

        let bot = Bot::new(env::var("BOT_TOKEN")?);

        Ok(
            Self {
                db,
                bot,
                chat_id,
                tx,
                vote: Arc::new(Mutex::new(VoteState::NotStarted))
            }
        )
    }
}

pub enum VoteState {
    NotStarted,
    InProcess(Vec<project::Model>),
}

impl VoteState {
    pub fn is_waiting(&self) -> bool {
        match self {
            VoteState::NotStarted => {
                true
            }
            _ => {
                false
            }
        }
    }

    pub fn get_projects(&self) -> Vec<project::Model> {
        match self {
            VoteState::InProcess(projects) => {
                projects.clone()
            }
            _ => {
                vec![]
            }
        }
    }
}