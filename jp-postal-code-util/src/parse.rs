use jp_postal_code_core::model::UtfKenAllRecord;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),

    #[error(transparent)]
    CsvError(#[from] csv::Error),
}

/// 指定された `reader` を `utf_ken_all.zip` としてパースし、郵便番号レコードを返す
///
/// `utf_ken_all.zip` に関しては [住所の郵便番号（1レコード1行、UTF-8形式）（CSV形式）] を参照。
///
/// 内部的には `utf_ken_all.zip` は `utf_ken_all.csv` を zip で圧縮したものとして扱われる（最初
/// のファイルが `utf_ken_all.csv` として扱われる）。
///
/// # Example
/// ```rust
/// # use jp_postal_code_util::parse_utf_ken_all_zip;
/// # fn main() {
/// # let ken_all_zip_file: std::path::PathBuf = vec![
/// #   env!("CARGO_MANIFEST_DIR"),
/// #   "testdata",
/// #   "partial_utf_ken_all.zip",
/// # ].iter().collect();
/// let ken_all_zip: std::fs::File = std::fs::File::open(ken_all_zip_file).unwrap();
/// let records = parse_utf_ken_all_zip(ken_all_zip).unwrap();
/// # }
/// ```
///
/// [住所の郵便番号（1レコード1行、UTF-8形式）（CSV形式）]: https://www.post.japanpost.jp/zipcode/dl/utf-zip.html
#[tracing::instrument(skip(reader))]
pub fn parse_utf_ken_all_zip<R>(reader: R) -> Result<Vec<UtfKenAllRecord>, ParseError>
where
    R: std::io::Read + std::io::Seek,
{
    tracing::info!("Start extracting `utf_ken_all.zip`");
    let mut zip = zip::ZipArchive::new(reader)?;
    let file = zip.by_index(0)?;
    tracing::info!(
        name = file.name(),
        compression = ?file.compression(),
        size = file.size(),
        last_modified = ?file.last_modified(),
        version_made_by = ?file.version_made_by(),
        "Finish extracting `utf_ken_all.zip`"
    );
    parse_utf_ken_all_csv(file).map_err(Into::into)
}

/// 指定された `reader` を `utf_ken_all.csv` としてパースし、郵便番号レコードを返す
///
/// `utf_ken_all.csv` の仕様は [郵便番号データ（1レコード1行、UTF-8形式）の説明] を参照
///
/// # Example
/// ```rust
/// # use jp_postal_code_util::parse_utf_ken_all_csv;
/// # fn main() {
/// # let ken_all_csv_file: std::path::PathBuf = vec![
/// #   env!("CARGO_MANIFEST_DIR"),
/// #   "testdata",
/// #   "partial_utf_ken_all.csv",
/// # ].iter().collect();
/// let ken_all_csv: std::fs::File = std::fs::File::open(ken_all_csv_file).unwrap();
/// let records = parse_utf_ken_all_csv(ken_all_csv).unwrap();
/// # }
/// ```
///
/// [郵便番号データ（1レコード1行、UTF-8形式）の説明]: https://www.post.japanpost.jp/zipcode/dl/utf-readme.html
#[tracing::instrument(skip(reader))]
pub fn parse_utf_ken_all_csv<R>(reader: R) -> Result<Vec<UtfKenAllRecord>, csv::Error>
where
    R: std::io::Read,
{
    tracing::info!("Start parsing `utf_ken_all.csv`");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);
    let result = rdr
        .deserialize::<UtfKenAllRecord>()
        .collect::<Result<Vec<_>, _>>()?;
    tracing::info!(count = result.len(), "Finish parsing `utf_ken_all.csv`");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_utf_ken_all_zip() {
        // テスト用にデータを削った partial_utf_ken_all.zip を利用
        let ken_all_zip: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "testdata",
            "partial_utf_ken_all.zip",
        ]
        .iter()
        .collect();
        let file = std::fs::File::open(ken_all_zip).unwrap();
        let records = parse_utf_ken_all_zip(file).unwrap();
        insta::assert_debug_snapshot!(records);
    }

    #[test]
    fn test_parse_utf_ken_all_csv() {
        // テスト用にデータを削った partial_utf_ken_all.csv を利用
        let ken_all_csv: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "testdata",
            "partial_utf_ken_all.csv",
        ]
        .iter()
        .collect();
        let file = std::fs::File::open(ken_all_csv).unwrap();
        let records = parse_utf_ken_all_csv(file).unwrap();
        insta::assert_debug_snapshot!(records);
    }
}
