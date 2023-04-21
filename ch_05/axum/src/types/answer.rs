use serde::{Deserialize, Serialize};

use crate::types::question::QuestionId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct AnswerId(pub String);
