use crate::repo::UtfKenAllRepository;
use jp_postal_code_core::model::UtfKenAllRecord;
use jp_postal_code_core::normalize::normalize_utf_ken_all_record_town;
use jp_postal_code_util::{download, parse_utf_ken_all_zip, UTF_KEN_ALL_URL};

/// 郵便番号データベースを更新する
#[tracing::instrument(skip(repo, utf_ken_all_zip_url))]
pub async fn update_postal_code_database<R, S>(
    repo: &mut R,
    utf_ken_all_zip_url: Option<S>,
) -> Result<(), anyhow::Error>
where
    R: UtfKenAllRepository,
    S: Into<String>,
{
    let utf_ken_all_zip_url = {
        if let Some(utf_ken_all_zip_url) = utf_ken_all_zip_url {
            utf_ken_all_zip_url.into()
        } else {
            UTF_KEN_ALL_URL.into()
        }
    };
    let mut tempfile = tempfile::tempfile()?;
    tracing::info!(
        ?utf_ken_all_zip_url,
        ?tempfile,
        "Download utf_ken_all.zip into a temp file"
    );
    download(utf_ken_all_zip_url, &mut tempfile).await?;
    tracing::info!(?tempfile, "Parse utf_ken_all.zip to records");
    let records = parse_utf_ken_all_zip(tempfile)?;
    tracing::info!(
        record_count = records.len(),
        "Replace database with the new records"
    );
    repo.replace(&records).await?;
    Ok(())
}

/// 郵便番号を検索する
#[tracing::instrument(skip(repo, postal_code))]
pub async fn search_postal_code<R>(
    repo: &R,
    postal_code: impl AsRef<str>,
) -> Result<Vec<UtfKenAllRecord>, anyhow::Error>
where
    R: UtfKenAllRepository,
{
    let postal_code = postal_code.as_ref();
    let records = repo.search(postal_code).await?;
    let records = records
        .into_iter()
        .flat_map(|r| {
            let towns = normalize_utf_ken_all_record_town(&r);
            towns
                .into_iter()
                .map(|town| UtfKenAllRecord { town, ..r.clone() })
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::ephemeral::UtfKenAllRepositoryEphemeral;

    #[tokio::test]
    async fn test_update_utf_ken_all_database() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/zipcode/dl/utf/zip/utf_ken_all.zip")
            .with_status(200)
            .with_header("content-type", "application/zip")
            .with_body_from_file("./testdata/partial_utf_ken_all.zip")
            .create_async()
            .await;

        let mut repo = UtfKenAllRepositoryEphemeral::default();
        update_postal_code_database(
            &mut repo,
            Some(server.url() + "/zipcode/dl/utf/zip/utf_ken_all.zip"),
        )
        .await
        .unwrap();
        mock.assert_async().await;
        insta::assert_debug_snapshot!(repo.into_inner().lock().unwrap());
    }

    #[tokio::test]
    async fn test_search_postal_code() {
        let repo = UtfKenAllRepositoryEphemeral::new(vec![
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
        let records = search_postal_code(&repo, "064").await.unwrap();
        insta::assert_debug_snapshot!(records);
    }
}
