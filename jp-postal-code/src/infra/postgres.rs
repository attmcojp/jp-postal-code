use crate::repo::UtfKenAllRepository;
use jp_postal_code_core::model::UtfKenAllRecord;
use sqlx::Connection as _;

#[derive(Debug, Clone)]
pub struct UtfKenAllRepositoryPostgres {
    pool: sqlx::PgPool,
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
        sqlx::query!("DELETE FROM utf_ken_all")
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
    async fn search(&self, postal_code: &str) -> Result<Vec<UtfKenAllRecord>, Self::Error> {
        tracing::info!(
            %postal_code,
            "Start finding records from utf_ken_all table"
        );
        let mut conn = self.pool.acquire().await?;
        let records = sqlx::query_as!(
            UtfKenAllRecord,
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
            WHERE postal_code LIKE $1
            "#,
            format!("{}%", postal_code)
        )
        .fetch_all(&mut *conn)
        .await?;
        tracing::info!(
            count = records.len(),
            "Finish finding records from utf_ken_all table"
        );
        Ok(records)
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
        let records = repository.search("0640820").await.unwrap();
        insta::assert_debug_snapshot!(records);

        // 前方一致で検索
        let records = repository.search("060").await.unwrap();
        insta::assert_debug_snapshot!(records);
    }
}
