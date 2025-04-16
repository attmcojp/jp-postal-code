use jp_postal_code_core::model::UtfKenAllRecord;

pub const DEFAULT_SEARCH_PAGE_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct UtfKenAllRepositorySearchRequest<'a> {
    pub postal_code: &'a str,
    pub page_size: Option<usize>,
    pub page_token: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct UtfKenAllRepositorySearchResponse {
    pub records: Vec<UtfKenAllRecord>,
    pub next_page_token: Option<String>,
}

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
        req: UtfKenAllRepositorySearchRequest<'_>,
    ) -> impl std::future::Future<Output = Result<UtfKenAllRepositorySearchResponse, Self::Error>> + Send;

    /// 郵便番号データベースの総数をカウントする
    fn count(&self) -> impl std::future::Future<Output = Result<usize, Self::Error>> + Send;
}
