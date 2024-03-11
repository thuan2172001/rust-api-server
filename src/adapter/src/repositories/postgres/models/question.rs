use std::time::SystemTime;

use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use rust_core::common::errors::CoreError;
use rust_core::entities::question::QuestionEntity;
use serde::Serialize;

#[derive(Debug, Queryable, Serialize, Selectable, Insertable, AsChangeset, Identifiable)]
#[diesel(table_name = super::super::schema::questions)]
#[cfg_attr(feature = "postgres", derive(diesel::pg::Pg))]
pub struct QuestionModel {
    pub id: i32,
    /// Title of the question.
    pub title: String,
    /// Content of the question.
    pub content: String,
    /// Optional tags associated with the question.
    pub tags: Option<Vec<Option<String>>>,

    pub created_on: SystemTime,
}

impl QuestionModel {
    pub fn from(entity: QuestionEntity) -> Result<Self, CoreError> {
        Ok(QuestionModel {
            id: entity.id.to_string().parse()?,
            title: entity.title,
            content: entity.content,
            tags: entity.tags.map(|v| v.into_iter().map(Some).collect()),
            created_on: SystemTime::now(),
        })
    }

    pub fn to_entity(self) -> Result<QuestionEntity, CoreError> {
        Ok(QuestionEntity {
            id: self.id.to_string().parse()?,
            title: self.title,
            content: self.content,
            tags: self.tags.map(|v| v.into_iter().flatten().collect()),
        })
    }
}
