use jp_postal_code_core::model::UtfKenAllRecord;

/// 郵便番号データベースを扱うリポジトリ
pub trait UtfKenAllRepository: Clone + Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// 郵便番号データベースを置き換える
    fn replace(
        &mut self,
        records: &[UtfKenAllRecord],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// 郵便番号データベースから前方一致でレコードを検索する
    fn search(
        &self,
        postal_code: &str,
    ) -> impl std::future::Future<Output = Result<Vec<UtfKenAllRecord>, Self::Error>> + Send;

    /// 郵便番号データベースの総数をカウントする
    fn count(&self) -> impl std::future::Future<Output = Result<usize, Self::Error>> + Send;
}
