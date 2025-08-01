use crate::repo::{
    UtfKenAllRepository, UtfKenAllRepositorySearchRequest, UtfKenAllRepositorySearchResponse,
    DEFAULT_SEARCH_PAGE_SIZE,
};
use jp_postal_code_core::model::UtfKenAllRecord;
use sqlx::Connection as _;

#[derive(Debug, Clone)]
pub struct UtfKenAllRepositoryPostgres {
    pool: sqlx::PgPool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PageToken {
    utf_ken_all_id: i64,
}

impl PageToken {
    fn new(utf_ken_all_id: i64) -> Self {
        Self { utf_ken_all_id }
    }

    fn parse(token: &str) -> Option<Self> {
        let decoded = base64_url::decode(token).ok()?;
        serde_json::from_slice::<PageToken>(&decoded).ok()
    }
}

impl std::fmt::Display for PageToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoded = serde_json::to_vec(self).expect("failed to JSON serialize PageToken");
        let token = base64_url::encode(&encoded);
        write!(f, "{token}")
    }
}

impl UtfKenAllRepositoryPostgres {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl UtfKenAllRepository for UtfKenAllRepositoryPostgres {
    type Error = sqlx::Error;

    #[tracing::instrument(skip(self, records))]
    async fn replace(&mut self, records: &[UtfKenAllRecord]) -> Result<(), Self::Error> {
        tracing::info!(
            count = records.len(),
            "Start inserting records into utf_ken_all table"
        );
        let updated_at = chrono::Utc::now();
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        // まず全レコードを削除
        sqlx::query!("TRUNCATE TABLE utf_ken_all RESTART IDENTITY")
            .execute(&mut *tx)
            .await?;
        // 次に指定されたレコードを全て挿入
        let mut query_builder: sqlx::QueryBuilder<sqlx::postgres::Postgres> =
            sqlx::QueryBuilder::new(
                r#"
                INSERT INTO utf_ken_all (
                    local_government_code,
                    old_postal_code,
                    postal_code,
                    prefecture_kana,
                    city_kana,
                    town_kana,
                    prefecture,
                    city,
                    town,
                    has_multi_postal_code,
                    has_chome,
                    has_multi_town,
                    update_code,
                    update_reason,
                    updated_at
                ) "#,
            );
        // https://github.com/launchbadge/sqlx/issues/3464
        const BIND_LIMIT: usize = u16::MAX as usize;
        for chunk in records.chunks(BIND_LIMIT / 15) {
            query_builder.reset();
            query_builder.push_values(chunk, |mut b, r| {
                b.push_bind(r.local_government_code.to_owned())
                    .push_bind(r.old_postal_code.to_owned())
                    .push_bind(r.postal_code.to_owned())
                    .push_bind(r.prefecture_kana.to_owned())
                    .push_bind(r.city_kana.to_owned())
                    .push_bind(r.town_kana.to_owned())
                    .push_bind(r.prefecture.to_owned())
                    .push_bind(r.city.to_owned())
                    .push_bind(r.town.to_owned())
                    .push_bind(r.has_multi_postal_code)
                    .push_bind(r.has_chome)
                    .push_bind(r.has_multi_town)
                    .push_bind(r.update_code)
                    .push_bind(r.update_reason)
                    .push_bind(updated_at);
            });
            query_builder.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        tracing::info!("Finish inserting records into utf_ken_all table");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn search(
        &self,
        req: UtfKenAllRepositorySearchRequest<'_>,
    ) -> Result<UtfKenAllRepositorySearchResponse, Self::Error> {
        let postal_code = req.postal_code;
        let page_size = req.page_size.unwrap_or(DEFAULT_SEARCH_PAGE_SIZE);
        let page_token = req.page_token.and_then(PageToken::parse);
        tracing::info!(
            %postal_code,
            "Start finding records from utf_ken_all table"
        );
        let mut conn = self.pool.acquire().await?;
        let mut records = if let Some(page_token) = page_token {
            sqlx::query_as!(
                DbUtfKenAllRecord,
                r#"
                SELECT
                    utf_ken_all_id,
                    local_government_code,
                    old_postal_code,
                    postal_code,
                    prefecture_kana,
                    city_kana,
                    town_kana,
                    prefecture,
                    city,
                    town,
                    has_multi_postal_code,
                    has_chome,
                    has_multi_town,
                    update_code,
                    update_reason
                FROM utf_ken_all
                WHERE utf_ken_all_id >= $3 AND postal_code LIKE $1
                ORDER BY postal_code, town, town_kana
                LIMIT $2
                "#,
                format!("{}%", postal_code),
                (page_size + 1) as i64,
                page_token.utf_ken_all_id,
            )
            .fetch_all(&mut *conn)
            .await?
        } else {
            sqlx::query_as!(
                DbUtfKenAllRecord,
                r#"
                SELECT
                    utf_ken_all_id,
                    local_government_code,
                    old_postal_code,
                    postal_code,
                    prefecture_kana,
                    city_kana,
                    town_kana,
                    prefecture,
                    city,
                    town,
                    has_multi_postal_code,
                    has_chome,
                    has_multi_town,
                    update_code,
                    update_reason
                FROM utf_ken_all
                WHERE postal_code LIKE $1
                ORDER BY postal_code, town, town_kana
                LIMIT $2
                "#,
                format!("{}%", postal_code),
                (page_size + 1) as i64,
            )
            .fetch_all(&mut *conn)
            .await?
        };
        tracing::info!(
            count = records.len(),
            "Finish finding records from utf_ken_all table"
        );
        let next_page_token = if records.len() > page_size {
            let utf_ken_all_id = records
                .pop()
                .expect("records should not be empty")
                .utf_ken_all_id;
            Some(PageToken::new(utf_ken_all_id).to_string())
        } else {
            None
        };
        let records = records
            .into_iter()
            .map(UtfKenAllRecord::from)
            .collect::<Vec<_>>();
        Ok(UtfKenAllRepositorySearchResponse {
            next_page_token,
            records,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn count(&self) -> Result<usize, Self::Error> {
        let mut conn = self.pool.acquire().await?;
        let count: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM utf_ken_all"#)
            .fetch_one(&mut *conn)
            .await?
            .unwrap_or(0);
        Ok(count as usize)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct DbUtfKenAllRecord {
    utf_ken_all_id: i64,
    local_government_code: String,
    old_postal_code: String,
    postal_code: String,
    prefecture_kana: String,
    city_kana: String,
    town_kana: String,
    prefecture: String,
    city: String,
    town: String,
    has_multi_postal_code: i16,
    has_chome: i16,
    has_multi_town: i16,
    update_code: i16,
    update_reason: i16,
}

impl From<DbUtfKenAllRecord> for UtfKenAllRecord {
    fn from(record: DbUtfKenAllRecord) -> Self {
        UtfKenAllRecord {
            local_government_code: record.local_government_code,
            old_postal_code: record.old_postal_code,
            postal_code: record.postal_code,
            prefecture_kana: record.prefecture_kana,
            city_kana: record.city_kana,
            town_kana: record.town_kana,
            prefecture: record.prefecture,
            city: record.city,
            town: record.town,
            has_multi_postal_code: record.has_multi_postal_code,
            has_chome: record.has_chome,
            has_multi_town: record.has_multi_town,
            update_code: record.update_code,
            update_reason: record.update_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MIGRATOR;

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn utf_ken_all_repository_postgres_replace(pool: sqlx::PgPool) {
        let mut repository = UtfKenAllRepositoryPostgres { pool: pool.clone() };

        // 正しくデータが挿入されているかチェック
        repository
            .replace(&[
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600000".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "イカニケイサイガナイバアイ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "以下に掲載がない場合".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 0,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640941".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "アサヒガオカ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "旭ケ丘".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600041".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリヒガシ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通東".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600042".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（１−１９チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（１〜１９丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640820".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（２０−２８チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（２０〜２８丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
            ])
            .await
            .unwrap();
        let records = {
            let mut conn = pool.acquire().await.unwrap();
            sqlx::query(
                r#"
                SELECT
                    local_government_code,
                    old_postal_code,
                    postal_code,
                    prefecture_kana,
                    city_kana,
                    town_kana,
                    prefecture,
                    city,
                    town,
                    has_multi_postal_code,
                    has_chome,
                    has_multi_town,
                    update_code,
                    update_reason
                FROM utf_ken_all
                "#,
            )
            .fetch_all(&mut *conn)
            .await
            .unwrap()
        };
        insta::assert_debug_snapshot!(records);

        // 正しく置き換えられるかチェック
        repository
            .replace(&[
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640941".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "アサヒガオカ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "旭ケ丘".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600042".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（１−１９チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（１〜１９丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
            ])
            .await
            .unwrap();
        let records = {
            let mut conn = pool.acquire().await.unwrap();
            sqlx::query(
                r#"
                SELECT
                    local_government_code,
                    old_postal_code,
                    postal_code,
                    prefecture_kana,
                    city_kana,
                    town_kana,
                    prefecture,
                    city,
                    town,
                    has_multi_postal_code,
                    has_chome,
                    has_multi_town,
                    update_code,
                    update_reason
                FROM utf_ken_all
                "#,
            )
            .fetch_all(&mut *conn)
            .await
            .unwrap()
        };
        insta::assert_debug_snapshot!(records);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn utf_ken_all_repository_postgres_search(pool: sqlx::PgPool) {
        let mut repository = UtfKenAllRepositoryPostgres { pool: pool.clone() };

        // サンプルデータを入力
        repository
            .replace(&[
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600000".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "イカニケイサイガナイバアイ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "以下に掲載がない場合".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 0,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640941".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "アサヒガオカ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "旭ケ丘".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600041".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリヒガシ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通東".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600042".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（１−１９チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（１〜１９丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640820".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（２０−２８チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（２０〜２８丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
            ])
            .await
            .unwrap();

        // 完全一致で検索
        let response = repository
            .search(UtfKenAllRepositorySearchRequest {
                postal_code: "0640820",
                page_size: None,
                page_token: None,
            })
            .await
            .unwrap();
        insta::assert_debug_snapshot!(response);

        // 前方一致で検索
        let response = repository
            .search(UtfKenAllRepositorySearchRequest {
                postal_code: "060",
                page_size: None,
                page_token: None,
            })
            .await
            .unwrap();
        insta::assert_debug_snapshot!(response);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn utf_ken_all_repository_postgres_search_page_size_and_page_token(pool: sqlx::PgPool) {
        let mut repository = UtfKenAllRepositoryPostgres { pool: pool.clone() };

        // サンプルデータを入力
        repository
            .replace(&[
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600000".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "イカニケイサイガナイバアイ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "以下に掲載がない場合".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 0,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640941".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "アサヒガオカ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "旭ケ丘".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600041".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリヒガシ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通東".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600042".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（１−１９チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（１〜１９丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640820".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（２０−２８チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（２０〜２８丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
            ])
            .await
            .unwrap();

        // 1, 2 件目を取得
        let response = repository
            .search(UtfKenAllRepositorySearchRequest {
                postal_code: "060",
                page_size: Some(2),
                page_token: None,
            })
            .await
            .unwrap();
        insta::assert_debug_snapshot!(response);

        // 3 件目を取得
        let response = repository
            .search(UtfKenAllRepositorySearchRequest {
                postal_code: "060",
                page_size: Some(2),
                page_token: response.next_page_token.as_deref(),
            })
            .await
            .unwrap();
        insta::assert_debug_snapshot!(response);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn utf_ken_all_repository_postgres_count(pool: sqlx::PgPool) {
        let mut repository = UtfKenAllRepositoryPostgres { pool: pool.clone() };

        let count = repository.count().await.unwrap();
        assert_eq!(count, 0);

        // サンプルデータを入力
        repository
            .replace(&[
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600000".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "イカニケイサイガナイバアイ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "以下に掲載がない場合".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 0,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640941".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "アサヒガオカ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "旭ケ丘".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600041".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリヒガシ".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通東".to_string(),
                    has_multi_postal_code: 0,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "060  ".to_string(),
                    postal_code: "0600042".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（１−１９チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（１〜１９丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
                UtfKenAllRecord {
                    local_government_code: "01101".to_string(),
                    old_postal_code: "064  ".to_string(),
                    postal_code: "0640820".to_string(),
                    prefecture_kana: "ホッカイドウ".to_string(),
                    city_kana: "サッポロシチュウオウク".to_string(),
                    town_kana: "オオドオリニシ（２０−２８チョウメ）".to_string(),
                    prefecture: "北海道".to_string(),
                    city: "札幌市中央区".to_string(),
                    town: "大通西（２０〜２８丁目）".to_string(),
                    has_multi_postal_code: 1,
                    has_chome: 0,
                    has_multi_town: 1,
                    update_code: 0,
                    update_reason: 0,
                },
            ])
            .await
            .unwrap();

        let count = repository.count().await.unwrap();
        assert_eq!(count, 5);
    }
}
