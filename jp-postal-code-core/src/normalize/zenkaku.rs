use std::sync::LazyLock;

/// 全角数字キャラクターか判定する
pub fn is_zenkaku_numeric(c: &char) -> bool {
    matches!(*c, '０'..='９')
}

/// 文字列から全角数字を抽出する
pub fn extract_zenkaku_numeric(s: &str) -> String {
    s.chars()
        .skip_while(|c| !is_zenkaku_numeric(c))
        .take_while(is_zenkaku_numeric)
        .collect()
}

/// 全角数字列を数値に変換する
pub fn from_zenkaku_numeric(s: &str) -> usize {
    let n: String = extract_zenkaku_numeric(s)
        .chars()
        .map(|c| match c {
            '０' => '0',
            '１' => '1',
            '２' => '2',
            '３' => '3',
            '４' => '4',
            '５' => '5',
            '６' => '6',
            '７' => '7',
            '８' => '8',
            '９' => '9',
            _ => unreachable!(),
        })
        .collect();
    n.parse()
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", s, e))
}

/// 数値を全角数字列に変換する
pub fn to_zenkaku_numeric(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|c| match c {
            '0' => '０',
            '1' => '１',
            '2' => '２',
            '3' => '３',
            '4' => '４',
            '5' => '５',
            '6' => '６',
            '7' => '７',
            '8' => '８',
            '9' => '９',
            _ => unreachable!(),
        })
        .collect()
}

/// 全角で表現されたラベル付きの範囲をリスト化する
pub fn zenkaku_range_label(start: &str, end: &str) -> Vec<String> {
    static PATTERN: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"[０-９]+").unwrap());
    let s = from_zenkaku_numeric(start);
    let e = from_zenkaku_numeric(end);
    (s..=e)
        .map(|n| PATTERN.replace(start, to_zenkaku_numeric(n)).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zenkaku_numeric() {
        assert!(is_zenkaku_numeric(&'０'));
        assert!(is_zenkaku_numeric(&'１'));
        assert!(is_zenkaku_numeric(&'２'));
        assert!(is_zenkaku_numeric(&'３'));
        assert!(is_zenkaku_numeric(&'４'));
        assert!(is_zenkaku_numeric(&'５'));
        assert!(is_zenkaku_numeric(&'６'));
        assert!(is_zenkaku_numeric(&'７'));
        assert!(is_zenkaku_numeric(&'８'));
        assert!(is_zenkaku_numeric(&'９'));
        assert!(!is_zenkaku_numeric(&'あ'));
        assert!(!is_zenkaku_numeric(&'ア'));
        assert!(!is_zenkaku_numeric(&'一'));
        assert!(!is_zenkaku_numeric(&'1'));
        assert!(!is_zenkaku_numeric(&'a'));
    }

    #[test]
    fn test_extract_zenkaku_numeric() {
        assert_eq!(extract_zenkaku_numeric("１２３"), "１２３");
        assert_eq!(extract_zenkaku_numeric("１２３abc"), "１２３");
        assert_eq!(extract_zenkaku_numeric("abc１２３"), "１２３");
        assert_eq!(extract_zenkaku_numeric("abc１２３abc"), "１２３");
        assert_eq!(extract_zenkaku_numeric("abc１２３abc４５６"), "１２３");
    }

    #[test]
    fn test_from_zenkaku_numeric() {
        assert_eq!(from_zenkaku_numeric("１２３"), 123);
        assert_eq!(from_zenkaku_numeric("１２３abc"), 123);
        assert_eq!(from_zenkaku_numeric("abc１２３"), 123);
        assert_eq!(from_zenkaku_numeric("abc１２３abc"), 123);
        assert_eq!(from_zenkaku_numeric("abc１２３abc４５６"), 123);
    }

    #[test]
    fn test_to_zenkaku_numeric() {
        assert_eq!(to_zenkaku_numeric(123), "１２３");
    }

    #[test]
    fn test_range_label() {
        assert_eq!(zenkaku_range_label("１", "３"), vec!["１", "２", "３"]);
        assert_eq!(
            zenkaku_range_label("８丁目", "１２丁目"),
            vec!["８丁目", "９丁目", "１０丁目", "１１丁目", "１２丁目"]
        );
        assert_eq!(
            zenkaku_range_label("第８地割", "第１２地割"),
            vec![
                "第８地割",
                "第９地割",
                "第１０地割",
                "第１１地割",
                "第１２地割"
            ]
        );
    }
}
