use chrono;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, prelude::FromRow, Decode, Encode, PgPool, Postgres, Type};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub enum Role {
    #[default]
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

impl Type<Postgres> for Role {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("role")
    }
}

impl<'q, DB> Encode<'q, DB> for Role
where
    DB: sqlx::Database,
    &'q str: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let value = match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
        };

        value.encode_by_ref(buf)
    }
}

impl<'r, DB> Decode<'r, DB> for Role
where
    DB: sqlx::Database,
    String: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = String::decode(value)?;

        match value.as_str() {
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "system" => Ok(Role::System),
            _ => Err("Unknown role".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Message {
    pub id: String,
    pub chat_id: i64,
    pub content: String,
    pub role: Role,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Default for Message {
    fn default() -> Self {
        let id = uuid::Uuid::now_v7().to_string();

        Self {
            id,
            chat_id: 0,
            content: "".to_string(),
            role: Role::User,
            created_at: chrono::Utc::now(),
        }
    }
}

impl Message {
    pub async fn create(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
      INSERT INTO messages (id, chat_id, content, role, created_at)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id
      "#,
        )
        .bind(&self.id)
        .bind(self.chat_id)
        .bind(&self.content)
        .bind(&self.role)
        .bind(self.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_chat_id(
        chat_id: i64,
        size: i32,
        pool: &PgPool,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
      SELECT id, chat_id, content, role, created_at
      FROM messages
      WHERE chat_id = $1
      ORDER BY created_at DESC
      LIMIT $2
      "#,
        )
        .bind(chat_id)
        .bind(size)
        .fetch_all(pool)
        .await?;

        Ok(messages)
    }
}
