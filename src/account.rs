use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: String,
    pub chat_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub verified: bool,
    pub did_onboarding: bool,
}

impl Default for Account {
    fn default() -> Self {
        let id = Uuid::now_v7().to_string();

        Self {
            id,
            chat_id: 0,
            first_name: "".to_string(),
            last_name: "".to_string(),
            verified: false,
            did_onboarding: false,
        }
    }
}

impl Account {
    pub async fn create(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO accounts (id, chat_id, first_name, last_name, verified, did_onboarding)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(&self.id)
        .bind(self.chat_id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(self.verified)
        .bind(self.did_onboarding)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE accounts
            SET chat_id = $1, first_name = $2, last_name = $3, verified = $4, did_onboarding = $5
            WHERE id = $6
            "#,
        )
        .bind(self.chat_id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(self.verified)
        .bind(self.did_onboarding)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_chat_id(
        chat_id: &i64,
        pool: &PgPool,
    ) -> Result<Option<Account>, sqlx::Error> {
        let account = sqlx::query_as(
            r#"
            SELECT id, chat_id, first_name, last_name, verified, did_onboarding
            FROM accounts
            WHERE chat_id = $1
            "#,
        )
        .bind(chat_id)
        .fetch_optional(pool)
        .await?;

        Ok(account)
    }
}
