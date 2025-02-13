/// 郵便番号レコード
///
/// オリジナルの情報を残すようなモデルにしている。
///
/// # Example
/// ```rust
/// # use jp_postal_code_core::model::UtfKenAllRecord;
/// let _ = UtfKenAllRecord {
///     local_government_code: "01101".to_string(),
///     old_postal_code: "060  ".to_string(),
///     postal_code: "0600000".to_string(),
///     prefecture_kana: "ホッカイドウ".to_string(),
///     city_kana: "サッポロシチュウオウク".to_string(),
///     town_kana: "イカニケイサイガナイバアイ".to_string(),
///     prefecture: "北海道".to_string(),
///     city: "札幌市中央区".to_string(),
///     town: "以下に掲載がない場合".to_string(),
///     has_multi_postal_code: 0,
///     has_chome: 0,
///     has_multi_town: 0,
///     update_code: 0,
///     update_reason: 0,
/// };
/// ```
///
/// # References
/// - [郵便番号データ（1レコード1行、UTF-8形式）の説明](https://www.post.japanpost.jp/zipcode/dl/utf-readme.html)
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct UtfKenAllRecord {
    /// 全国地方公共団体コード（JIS X0401、X0402）
    pub local_government_code: String,
    /// 旧郵便番号（5桁）
    pub old_postal_code: String,
    /// 郵便番号（7桁）
    pub postal_code: String,
    /// 都道府県名（カタカナ）
    pub prefecture_kana: String,
    /// 市区町村名（カタカナ）
    pub city_kana: String,
    /// 町域名（カタカナ）
    pub town_kana: String,
    /// 都道府県名
    pub prefecture: String,
    /// 市区町村名
    pub city: String,
    /// 町域名
    pub town: String,
    /// 一町域が二以上の郵便番号で表される場合の表示
    ///
    /// - 0: 該当せず
    /// - 1: 該当
    pub has_multi_postal_code: i16,
    /// 丁目を有する町域の場合の表示
    ///
    /// - 0: 該当せず
    /// - 1: 該当
    pub has_chome: i16,
    /// 一つの郵便番号で二以上の町域を表す場合の表示
    ///
    /// - 0: 該当せず
    /// - 1: 該当
    pub has_multi_town: i16,
    /// 更新の表示
    ///
    /// - 0: 変更なし
    /// - 1: 変更あり
    /// - 2: 廃止
    pub update_code: i16,
    /// 更新理由
    ///
    /// - 0: 変更なし
    /// - 1: 市政・区政・町政・分区・政令指定都市施行
    /// - 2: 住居表示の実施
    /// - 3: 区画整理
    /// - 4: 郵便区調整等
    /// - 5: 訂正
    /// - 6: 廃止
    pub update_reason: i16,
}
