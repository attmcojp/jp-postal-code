use super::zenkaku;
use crate::model::UtfKenAllRecord;
use regex::Regex;
use std::sync::LazyLock;

/// 郵便番号レコードから正規化した町域を抽出する
pub fn normalize_utf_ken_all_record_town(record: &UtfKenAllRecord) -> Vec<String> {
    //
    // 町名・地番を削除するケース
    // このケースはアーリーリターンで終わらせる
    //
    if record.town == "以下に掲載がない場合" {
        // 6000000 北海道 札幌市中央区 以下に掲載がない場合
        // 4320000 静岡県 浜松市中区 以下に掲載がない場合
        return vec!["".to_string()];
    }
    if record.town.ends_with("の次に番地がくる場合") {
        // 3060433 茨城県 猿島郡境町 境町の次に番地がくる場合
        // 4150001 静岡県 下田市 下田市の次に番地がくる場合
        return vec!["".to_string()];
    }
    if record.town != "一円" && record.city.ends_with(record.town.trim_end_matches("一円")) {
        // 1000301 東京都 利島村 利島村一円
        // 3998501 長野県 北安曇郡松川村 松川村一円
        return vec!["".to_string()];
    }

    // 波ダッシュ（U+301C）と全角チルダ（U+FF5E）を全角チルダに統一
    let town = {
        define!(PATTERN, r"[〜～]");
        PATTERN.replace_all(&record.town, "～").to_string()
    };
    // ダッシュ系統を全角ハイフンに統一
    // — 全角ダッシュ（EMダッシュ）	U+2014
    // - ハイフン	                U+002d
    // − マイナス	                U+2212
    // – ENダッシュ	                U+2013
    // －全角ハイフン	            U+ff0d
    let mut town = {
        define!(PATTERN, r"[—-−–－]");
        PATTERN.replace_all(&town, "－").to_string()
    };

    //
    // 不要文字列の除去
    //
    {
        let patterns = [
            "（全域）",
            "（地階・階層不明）",
            "（次のビルを除く）",
            "（丁目）",
            "（各町）",
            "（番地）",
            "（無番地）",
            "（その他）",
        ];
        patterns.iter().for_each(|pat| {
            town = town.replace(pat, "");
        });
    }

    //
    // 屋敷の除去
    //
    {
        // 4411336 愛知県 新城市 富岡（○○屋敷）
        define!(PATTERN, r"（.*?屋敷）");
        town = PATTERN.replace_all(&town, "").to_string();
    }

    //
    // 階数から括弧を除去
    //
    {
        // 1066101 東京都港区六本木六本木ヒルズ森タワー（１階）
        // 9806101 宮城県 仙台市青葉区 中央アエル（１階）
        // 1506147 東京都 渋谷区 渋谷渋谷スクランブルスクエア（４７階）
        // 4506210 愛知県 名古屋市中村区 名駅ミッドランドスクエア（高層棟）（１０階）
        define!(PATTERN, r"（([０-９]+階)）");
        town = PATTERN.replace_all(&town, "$1").to_string();
    }

    //
    // 地番関連文字列の除去
    //
    {
        // 0580343 北海道 幌泉郡えりも町 東洋（油駒、南東洋、１３２～１５６、１５８～３５４、３６６、３６７番地）
        // 3812241 長野県 長野市 青木島町青木島乙（９５６番地以外）
        // 3998251 長野県 松本市 島内（９８２０、９８２１、９８２３〜９８３０、９８６４番地以上）
        // 6496413 和歌山県 紀の川市 竹房（４５０番地以下）
        // 8911274 鹿児島県 鹿児島市 緑ヶ丘町（３５番以降）
        // 9880927 宮城県 気仙沼市 唐桑町西舞根（２００番以上）
        // 9960301 山形県 最上郡大蔵村 南山（４３０番地以上「１７７０－１〜２、１８６２－４２、１９２３－５を除く」、大谷地、折渡、鍵金野、金山、滝ノ沢、豊牧、沼の台、肘折、平林）
        define!(
            PATTERN,
            r"(?:(?:[０-９]+[～－])?[０-９]+、)*(?:[０-９]+[～－])?[０-９]+番地?(?:以降|以下|以上|以外)?"
        );
        town = PATTERN.replace_all(&town, "").to_string();
    }
    {
        // 8830104 宮崎県 日向市 東郷町山陰戊（５１３の１以内）
        define!(PATTERN, r"（[０-９]+の[０-９]+以内）");
        town = PATTERN.replace_all(&town, "").to_string();
    }
    {
        // 9960301 山形県 最上郡大蔵村 南山（４３０番地以上「１７７０－１〜２、１８６２－４２、１９２３－５を除く」、大谷地、折渡、鍵金野、金山、滝ノ沢、豊牧、沼の台、肘折、平林）
        // 0285102 岩手県 岩手郡葛巻町 葛巻（第４０地割「５７番地１２５、１７６を除く」～第４５地割）
        define!(PATTERN, r"「.*?を除く」");
        town = PATTERN.replace_all(&town, "").to_string();
    }
    {
        // 0482402 北海道 余市郡仁木町 大江（１丁目、２丁目「６５１、６６２、６６８番地」以外、３丁目５、１３－４、２０、６７８、６８７番地）
        define!(PATTERN, r"「.*?」以外");
        town = PATTERN.replace_all(&town, "").to_string();
    }
    {
        // 2900156 千葉県 市原市 草刈（１６５６〜１９９９）
        define!(PATTERN, r"（(?:[０-９]+[～－])?[０-９]+）");
        town = PATTERN.replace_all(&town, "").to_string();
    }

    //
    // 地割後の補足文字列の除去
    //
    {
        // 0287917 岩手県 九戸郡洋野町 種市第５０地割〜第７０地割（大沢、城内、滝沢）
        define!(PATTERN, r"地割（.*?）");
        town = PATTERN.replace_all(&town, "地割").to_string();
    }

    //
    // 加工で発生したゴミの除去
    //
    {
        define!(PATTERN1, r"、+");
        town = PATTERN1.replace_all(&town, "、").to_string();
        define!(PATTERN2, r"（、");
        town = PATTERN2.replace_all(&town, "（").to_string();
        define!(PATTERN3, r"、）");
        town = PATTERN3.replace_all(&town, "）").to_string();
        define!(PATTERN4, r"（）");
        town = PATTERN4.replace_all(&town, "").to_string();
    }

    //
    // 甲、乙
    //
    if town.starts_with("甲、乙") {
        // 7614103 香川県 小豆郡土庄町 甲、乙（大木戸）
        define!(PATTERN, r"^(甲、乙.*?)（.*）$");
        town = PATTERN.replace_all(&town, "$1").to_string();
        define!(DELIMITER, r"[、・]");
        let towns: Vec<_> = DELIMITER.split(&town).collect();
        return towns.into_iter().map(Into::into).collect();
    }

    //
    // （Ａ～Ｂ丁目）
    //
    {
        // 2080032 東京都 武蔵村山市 三ツ木（１～５丁目）
        // 1080023 東京都 港区 芝浦（２〜４丁目）
        define!(PATTERN, r"（([０-９]+)[～－]([０-９]+)丁目）");
        if let Some(caps) = PATTERN.captures(&town) {
            let prefix = PATTERN.replace_all(&town, "").to_string();
            let mut suffixes = zenkaku::zenkaku_range_label(
                format!("{}{}", &caps[1], "丁目").as_str(),
                format!("{}{}", &caps[2], "丁目").as_str(),
            );
            // 括弧有り丁目の場合は親住所も加える
            suffixes.push("".to_string());
            return suffixes
                .iter()
                .map(|s| format!("{}{}", prefix, s))
                .collect();
        }
    }

    //
    // （第Ａ地割～第Ｂ地割）
    //
    {
        // 0285102 岩手県 岩手郡葛巻町 葛巻（第４０地割「５７番地１２５、１７６を除く」～第４５地割）
        define!(PATTERN, r"（(第[０-９]+地割)[～－](第[０-９]+地割)）");
        if let Some(caps) = PATTERN.captures(&town) {
            let prefix = PATTERN.replace_all(&town, "").to_string();
            let suffixes = zenkaku::zenkaku_range_label(&caps[1], &caps[2]);
            // 括弧有り地割の場合は親住所は加えない
            return suffixes
                .iter()
                .map(|s| format!("{}{}", prefix, s))
                .collect();
        }
    }

    //
    // 第Ａ地割～第Ｂ地割
    //
    {
        // 0287917 岩手県 九戸郡洋野町 種市第５０地割〜第７０地割（大沢、城内、滝沢）
        define!(PATTERN, r"(第[０-９]+地割)[～－](第[０-９]+地割)");
        if let Some(caps) = PATTERN.captures(&town) {
            let prefix = PATTERN.replace_all(&town, "").to_string();
            let mut suffixes = zenkaku::zenkaku_range_label(&caps[1], &caps[2]);
            // 括弧無し地割の場合は親住所も加える
            suffixes.push("".to_string());
            return suffixes
                .iter()
                .map(|s| format!("{}{}", prefix, s))
                .collect();
        }
    }

    //
    // ＸＡ地割～ＸＢ地割
    //
    {
        // 0295523 岩手県 和賀郡西和賀町 越中畑６４地割〜越中畑６６地割
        define!(PATTERN, r"((.*?)[０-９]+地割)[～－](.*?[０-９]+地割)");
        if let Some(caps) = PATTERN.captures(&town) {
            let prefix = PATTERN.replace_all(&town, "").to_string();
            let mut suffixes = zenkaku::zenkaku_range_label(&caps[1], &caps[3]);
            // 括弧無し地割の場合は親住所も加える
            suffixes.push(caps[2].to_string());
            return suffixes
                .iter()
                .map(|s| format!("{}{}", prefix, s))
                .collect();
        }
    }

    //
    // （Ａ・Ｂ・...）
    //
    {
        // 9401172 新潟県 長岡市 釜ケ島（土手畑・藤場）
        define!(PATTERN, r"（(.*?・.*?)）");
        if let Some(caps) = PATTERN.captures(&town) {
            let prefix = PATTERN.replace_all(&town, "").to_string();
            let inner = caps[1].to_string();
            let inner = inner.replace("、その他", "");
            let inner = inner.replace("を含む", "");
            let suffixes: Vec<_> = inner.split("・").collect();
            // 括弧有り中点列挙の場合は親住所を加えない
            return suffixes
                .iter()
                .map(|s| format!("{}{}", prefix, s))
                .collect();
        }
    }

    //
    // （Ａ、Ｂ、...）or（Ａを含む）
    //
    {
        // 0580343 北海道 幌泉郡えりも町 東洋（油駒、南東洋、１３２～１５６、１５８～３５４、３６６、３６７番地）
        // 6028064 京都府 京都市上京区 一町目（上長者町通堀川東入、東堀川通上長者町上る、東堀川通中立売通下る）
        // 0790177 北海道 美唄市 上美唄町（協和、南）
        // 0482402 北海道 余市郡仁木町 大江（１丁目、２丁目「６５１、６６２、６６８番地」以外、３丁目５、１３－４、２０、６７８、６８７番地）
        // 4400845 愛知県 豊橋市 高師町（北原、その他）
        // 7860301 高知県 高岡郡四万十町 大正（葛籠川、轟崎を含む）
        // 9960301 山形県 最上郡大蔵村 南山（４３０番地以上「１７７０－１〜２、１８６２－４２、１９２３－５を除く」、大谷地、折渡、鍵金野、金山、滝ノ沢、豊牧、沼の台、肘折、平林）
        define!(PATTERN, r"（(.*?、.*?|.*?を含む)）");
        if let Some(caps) = PATTERN.captures(&town) {
            let prefix = PATTERN.replace_all(&town, "").to_string();
            let inner = caps[1].to_string();
            let inner = inner.replace("、その他", "");
            let inner = inner.replace("を含む", "");
            let mut suffixes: Vec<_> = inner.split("、").collect();
            // 括弧有り句点列挙の場合は親住所も加える
            suffixes.push("");
            return suffixes
                .iter()
                .map(|s| format!("{}{}", prefix, s))
                .collect();
        }
    }

    //
    // Ａ、Ｂ、...
    //
    {
        // 0295503 岩手県 和賀郡西和賀町 穴明２２地割、穴明２３地割
        define!(PATTERN, r"(.*?、.*?)");
        if PATTERN.captures(&town).is_some() {
            let towns: Vec<_> = town.split("、").collect();
            // 括弧無し句点列挙の場合は親住所を加えない
            return towns.into_iter().map(Into::into).collect();
        }
    }

    //
    // 全ての処理が終わったはずなので、丸括弧と鉤括弧を除去する
    // ただし（高層棟）のような括弧は除去しない
    //
    {
        let skip_patterns = ["（高層棟）"];
        define!(PATTERN1, r"（(.*?)）");
        town = PATTERN1
            .replace_all(&town, |caps: &regex::Captures| -> String {
                let matched = caps.get(0).unwrap().as_str();
                if skip_patterns.contains(&matched) {
                    matched.to_string()
                } else {
                    let inner = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();
                    inner.to_string()
                }
            })
            .to_string();
        define!(PATTERN2, r"「(.*?)」");
        town = PATTERN2
            .replace_all(&town, |caps: &regex::Captures| -> String {
                let matched = caps.get(0).unwrap().as_str();
                if skip_patterns.contains(&matched) {
                    matched.to_string()
                } else {
                    let inner = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();
                    inner.to_string()
                }
            })
            .to_string();
    }

    vec![town]
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafile_test::datafile_test;
    use itertools::Itertools as _;

    #[derive(Debug, serde::Deserialize)]
    struct TestCase {
        source: String,
        result: Vec<String>,
    }

    impl TestCase {
        fn to_utf_ken_all_record(&self) -> UtfKenAllRecord {
            let (postal_code, prefecture, city, town) =
                self.source.split(' ').collect_tuple().unwrap_or_else(|| {
                    panic!("Invalid source: {}", self.source);
                });
            UtfKenAllRecord {
                local_government_code: "00000".to_string(),
                old_postal_code: "000  ".to_string(),
                postal_code: postal_code.to_string(),
                prefecture_kana: "".to_string(),
                city_kana: "".to_string(),
                town_kana: "".to_string(),
                prefecture: prefecture.to_string(),
                city: city.to_string(),
                town: town.to_string(),
                has_multi_postal_code: 0,
                has_chome: 0,
                has_multi_town: 0,
                update_code: 0,
                update_reason: 0,
            }
        }
    }

    #[datafile_test("./jp-postal-code-core/testdata/testcase.yml")]
    fn test_normalize_utf_ken_all_record(testcase: TestCase) {
        let source = testcase.to_utf_ken_all_record();
        let mut result = normalize_utf_ken_all_record_town(&source);
        let mut expect = testcase.result;
        result.sort();
        expect.sort();
        pretty_assertions::assert_eq!(result, expect);
    }
}
