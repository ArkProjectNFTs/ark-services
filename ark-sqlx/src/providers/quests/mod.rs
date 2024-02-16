use super::ProviderError;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tracing::trace;

#[derive(Debug, Clone, Deserialize)]
pub struct QuestToValidate {
    pub quest_type: String,
    pub account_address: String,
}

pub struct QuestProvider;

impl QuestProvider {
    pub async fn validate(
        pool: &Pool<Postgres>,
        quest: &QuestToValidate,
    ) -> Result<(), ProviderError> {
        trace!("Registering quest {:?}", quest);

        let q = r#"
        INSERT INTO public."Quest" ("questType", "accountAddress", "createdAt", "updatedAt", "count") 
        VALUES ($1, $2, NOW(), NOW(), 1)
        ON CONFLICT ("questType", "accountAddress") 
        DO UPDATE SET 
            "updatedAt" = NOW(), 
            "count" = public."Quest"."count" + 1
        WHERE public."Quest"."questType" = EXCLUDED."questType"
        AND public."Quest"."accountAddress" = EXCLUDED."accountAddress";
        "#;

        sqlx::query(q)
            .bind(&quest.quest_type)
            .bind(&quest.account_address)
            .execute(pool)
            .await?;

        Ok(())
    }
}
