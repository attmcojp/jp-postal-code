mod zenkaku;

// 正規表現の定義用マクロ
macro_rules! define {
    ($name:ident, $re:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($re).unwrap());
    };
}
#[macro_use]
mod town;

pub use town::normalize_utf_ken_all_record_town;
