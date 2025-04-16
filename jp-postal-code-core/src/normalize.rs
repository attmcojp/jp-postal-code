mod zenkaku;

// 正規表現の定義用マクロ
macro_rules! define {
    ($name:ident, $re:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($re).unwrap());
    };
}
#[macro_use]
mod town;
mod town_kana;

pub use town::normalize_utf_ken_all_record_town;
pub use town_kana::normalize_utf_ken_all_record_town_kana;
