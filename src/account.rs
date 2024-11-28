use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    async fn save(&self, pool: PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO account (chat_id, first_name, last_name, verified, did_onboarding)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (chat_id) DO UPDATE SET
                first_name = $2,
                last_name = $3,
                verified = $4,
                did_onboarding = $5
            "#,
        )
        .bind(self.chat_id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(self.verified)
        .bind(self.did_onboarding)
        .execute(pool)
        .await?;

        Ok(())
    }
}
