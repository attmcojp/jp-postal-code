use crate::repo::{
    UtfKenAllRepository, UtfKenAllRepositorySearchRequest, UtfKenAllRepositorySearchResponse,
    DEFAULT_SEARCH_PAGE_SIZE,
};
use jp_postal_code_core::model::UtfKenAllRecord;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct UtfKenAllRepositoryEphemeral {
    records: Arc<Mutex<Vec<UtfKenAllRecord>>>,
}

impl UtfKenAllRepositoryEphemeral {
    pub fn new(records: Vec<UtfKenAllRecord>) -> Self {
        Self {
            records: Arc::new(Mutex::new(records)),
        }
    }

    pub fn into_inner(self) -> Arc<Mutex<Vec<UtfKenAllRecord>>> {
        self.records
    }
}

impl UtfKenAllRepository for UtfKenAllRepositoryEphemeral {
    type Error = std::convert::Infallible;

    #[tracing::instrument(skip(self, records))]
    async fn replace(&mut self, records: &[UtfKenAllRecord]) -> Result<(), Self::Error> {
        self.records
            .lock()
            .unwrap()
            .splice(0.., records.iter().cloned());
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn search(
        &self,
        req: UtfKenAllRepositorySearchRequest<'_>,
    ) -> Result<UtfKenAllRepositorySearchResponse, Self::Error> {
        let page_size = req.page_size.unwrap_or(DEFAULT_SEARCH_PAGE_SIZE);
        let offset = req
            .page_token
            .and_then(|token| token.parse::<usize>().ok())
            .unwrap_or(0);
        let records = self
            .records
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.postal_code.starts_with(req.postal_code))
            .skip(offset)
            .take(page_size)
            .map(Clone::clone)
            .collect::<Vec<_>>();
        let next_page_token = if records.len() == page_size {
            Some((offset + page_size).to_string())
        } else {
            None
        };
        Ok(UtfKenAllRepositorySearchResponse {
            records,
            next_page_token,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn count(&self) -> Result<usize, Self::Error> {
        Ok(self.records.lock().unwrap().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn utf_ken_all_repository_ephemeral_replace() {
        let mut repository = UtfKenAllRepositoryEphemeral::default();

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
        insta::assert_debug_snapshot!(repository.records.lock().unwrap());

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
        insta::assert_debug_snapshot!(repository.records.lock().unwrap());
    }

    #[tokio::test]
    async fn utf_ken_all_repository_ephemeral_search() {
        let repository = UtfKenAllRepositoryEphemeral::new(vec![
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
        ]);

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

    #[tokio::test]
    async fn utf_ken_all_repository_ephemeral_search_page_size_and_page_token() {
        let repository = UtfKenAllRepositoryEphemeral::new(vec![
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
        ]);

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

    #[tokio::test]
    async fn utf_ken_all_repository_ephemeral_count() {
        let mut repository = UtfKenAllRepositoryEphemeral::new(vec![]);
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
