use futures::StreamExt as _;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// 日本郵便が配布している `utf_ken_all.zip` のダウンロード URL
///
/// `utf_ken_all.zip` に関しては [住所の郵便番号（1レコード1行、UTF-8形式）（CSV形式）] を参照。
/// 約 2 MB ほどのデータで、毎月末に情報が更新されている様子。
///
/// [住所の郵便番号（1レコード1行、UTF-8形式）（CSV形式）]: https://www.post.japanpost.jp/zipcode/dl/utf-zip.html
pub static UTF_KEN_ALL_URL: &str =
    "https://www.post.japanpost.jp/zipcode/dl/utf/zip/utf_ken_all.zip";

/// 指定された `url` の内容を指定された `writer` に書き込む
///
/// 日本郵便が配布している郵便番号データをダウンロードするには [UTF_KEN_ALL_URL] を指定する。
///
/// # Example
/// ```rust
/// # use jp_postal_code_util::{download, UTF_KEN_ALL_URL};
/// # #[tokio::main]
/// # async fn main() {
/// let mut writer = std::io::Cursor::new(vec![]);
/// download(UTF_KEN_ALL_URL, &mut writer).await.unwrap();
/// # }
/// ```
///
#[tracing::instrument(skip(url, writer))]
pub async fn download<U, W>(url: U, mut writer: W) -> Result<(), DownloadError>
where
    U: AsRef<str>,
    W: std::io::Write,
{
    let url = url.as_ref();
    tracing::info!(?url, "Downloading...",);
    let mut byte_stream = reqwest::get(url).await?.bytes_stream();
    while let Some(chunk) = byte_stream.next().await {
        std::io::copy(&mut chunk?.as_ref(), &mut writer)?;
    }
    tracing::info!(?url, "Downloaded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;

    #[tokio::test]
    async fn test_download() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/hello")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("world")
            .create_async()
            .await;

        let mut buffer = std::io::Cursor::new(Vec::new());
        download(server.url() + "/hello", &mut buffer)
            .await
            .unwrap();
        mock.assert_async().await;

        let mut data = String::new();
        buffer.set_position(0);
        buffer.read_to_string(&mut data).unwrap();
        assert_eq!(data, "world");
    }
}
