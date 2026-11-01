use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use crate::entity::public::project;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag="type", content="payload", rename_all="camelCase")]
pub enum VoteEvent {
    Close,
    InProcess {},
    Vote(i32, i32, i32),
    Start(Vec<project::Model>),
    Waiting,
    Empty
}

impl From<VoteEvent> for Message {
    fn from(value: VoteEvent) -> Self {
        to_string(&value).unwrap().into()
    }
}