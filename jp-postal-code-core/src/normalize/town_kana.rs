use super::zenkaku;
use crate::model::UtfKenAllRecord;
use regex::Regex;
use std::sync::LazyLock;

/// 郵便番号レコードから正規化した町域（仮名）を抽出する
pub fn normalize_utf_ken_all_record_town_kana(record: &UtfKenAllRecord) -> Vec<String> {
    //
    // 町名・地番を削除するケース
    // このケースはアーリーリターンで終わらせる
    //
    if record.town_kana == "イカニケイサイガナイバアイ" {
        // 6000000 キョウトフ キョウトシシモギョウク イカニケイサイガナイバアイ
        // 4320000 シズオカケン ハママツシチュウオウク イカニケイサイガナイバアイ
        return vec!["".to_string()];
    }
    if record.town_kana.ends_with("ノツギニバンチガクルバアイ") {
        // 3060433 イバラキケン サシマグンサカイマチ サカイマチノツギニバンチガクルバアイ
        // 4150001 シズオカケン シモダシ シモダシノツギニバンチガクルバアイ
        return vec!["".to_string()];
    }
    if record.town_kana != "イチエン"
        && record
            .city_kana
            .ends_with(record.town_kana.trim_end_matches("イチエン"))
    {
        // 1000301 トウキョウト トシマムラ トシマムライチエン
        // 3998501 ナガノケン キタアヅミグンマツカワムラ マツカワムライチエン
        return vec!["".to_string()];
    }

    // 波ダッシュ（U+301C）と全角チルダ（U+FF5E）を全角チルダに統一
    let town_kana = {
        define!(PATTERN, r"[〜～]");
        PATTERN.replace_all(&record.town_kana, "～").to_string()
    };
    // ダッシュ系統を全角ハイフンに統一
    // — 全角ダッシュ（EMダッシュ）	U+2014
    // - ハイフン	                U+002d
    // − マイナス	                U+2212
    // – ENダッシュ	                U+2013
    // －全角ハイフン	            U+ff0d
    let mut town_kana = {
        define!(PATTERN, r"[—-−–－]");
        PATTERN.replace_all(&town_kana, "－").to_string()
    };

    //
    // 不要文字列の除去
    //
    {
        let patterns = [
            "（ゼンイキ）",
            "（チカイ・カイソウフメイ）",
            "（ツギノビルヲノゾク）",
            "（チョウメ）",
            "（カクマチ）",
            "（バンチ）",
            "（ムバンチ）",
            "（ソノタ）",
        ];
        patterns.iter().for_each(|pat| {
            town_kana = town_kana.replace(pat, "");
        });
    }

    //
    // 屋敷の除去
    //
    {
        // 4411336 アイチケン シンシロシ トミオカ（ヤシキチク）
        define!(PATTERN, r"（.*?ヤシキチク）");
        town_kana = PATTERN.replace_all(&town_kana, "").to_string();
    }

    //
    // 階数から括弧を除去
    //
    {
        // 1066101 トウキョウト ミナトク ロッポンギロッポンギヒルズモリタワー（１カイ）
        // 9806101 ミヤギケン センダイシアオバク チュウオウアエル（１カイ）
        // 1506147 トウキョウト シブヤク シブヤシブヤスクランブルスクエア（４７カイ）
        // 4506210 アイチケン ナゴヤシナカムラク メイエキミッドランドスクエア（コウソウトウ）（１０カイ）
        define!(PATTERN, r"（([０-９]+カイ)）");
        town_kana = PATTERN.replace_all(&town_kana, "$1").to_string();
    }

    //
    // 地番関連文字列の除去
    //
    {
        // 0580343 ホッカイドウ ホロイズミグンエリモチョウ トウヨウ（アブラコマ、ミナミトウヨウ、１３２－１５６、１５８－３５４、３６６、３６７バンチ）
        // 3998251 ナガノケン マツモトシ シマウチ（９８２０、９８２１、９８２３－９８３０、９８６４バンチイジョウ）
        // 6496413 ワカヤマケン キノカワシ タケブサ（４５０バンチイカ）
        // 8911274 カゴシマケン カゴシマシ ミドリガオカチョウ（３５バンイコウ）
        // 9880927 ミヤギケン ケセンヌマシ カラクワチョウニシモウネ（２００バンイジョウ）
        // 9960301 ヤマガタケン モガミグンオオクラムラ ミナミヤマ（４３０バンチイジョウ＜１７７０－１－２、１８６２－４２、１９２３－５ヲノゾク＞、オオヤチ、オリワタリ、カンカネノ、キンザン、タキノサワ、トヨマキ、ヌマノダイ、ヒジオリ、ヒラバヤシ）
        define!(
            PATTERN,
            r"(?:(?:[０-９]+[～－])?[０-９]+、)*(?:[０-９]+[～－])?[０-９]+バンチ?(?:イコウ|イカ|イジョウ|イガイ)?"
        );
        town_kana = PATTERN.replace_all(&town_kana, "").to_string();
    }
    {
        // 8830104 ミヤザキケン ヒュウガシ トウゴウチョウヤマゲボ（５１３ノ１イナイ）
        define!(PATTERN, r"（[０-９]+ノ[０-９]+イナイ）");
        town_kana = PATTERN.replace_all(&town_kana, "").to_string();
    }
    {
        // 9960301 ヤマガタケン モガミグンオオクラムラ ミナミヤマ（４３０バンチイジョウ＜１７７０－１－２、１８６２－４２、１９２３－５ヲノゾク＞、オオヤチ、オリワタリ、カンカネノ、キンザン、タキノサワ、トヨマキ、ヌマノダイ、ヒジオリ、ヒラバヤシ）
        // 0285102 イワテケン イワテグンクズマキマチ クズマキ（ダイ４０チワリ＜５７バンチ１２５、１７６ヲノゾク＞－ダイ４５チワリ）
        define!(PATTERN, r"＜.*?ヲノゾク＞");
        town_kana = PATTERN.replace_all(&town_kana, "").to_string();
    }
    {
        // 0482402 ホッカイドウ ヨイチグンニキチョウ オオエ（１チョウメ、２チョウメ＜６５１、６６２、６６８バンチ＞イガイ、３チョウメ５、１３－４、２０、６７８、６８７バンチ）
        define!(PATTERN, r"＜.*?＞イガイ");
        town_kana = PATTERN.replace_all(&town_kana, "").to_string();
    }
    {
        // 2900156 チバケン イチハラシ クサカリ（１６５６－１９９９）
        define!(PATTERN, r"（(?:[０-９]+[～－])?[０-９]+）");
        town_kana = PATTERN.replace_all(&town_kana, "").to_string();
    }

    //
    // 地割後の補足文字列の除去
    //
    {
        // 0287917 イワテケン クノヘグンヒロノチョウ タネイチダイ５０チワリ－ダイ７０チワリ（オオサワ、ジョウナイ、タキサワ）
        define!(PATTERN, r"チワリ（.*?）");
        town_kana = PATTERN.replace_all(&town_kana, "チワリ").to_string();
    }

    //
    // 加工で発生したゴミの除去
    //
    {
        define!(PATTERN1, r"、+");
        town_kana = PATTERN1.replace_all(&town_kana, "、").to_string();
        define!(PATTERN2, r"（、");
        town_kana = PATTERN2.replace_all(&town_kana, "（").to_string();
        define!(PATTERN3, r"、）");
        town_kana = PATTERN3.replace_all(&town_kana, "）").to_string();
        define!(PATTERN4, r"（）");
        town_kana = PATTERN4.replace_all(&town_kana, "").to_string();
    }

    //
    // 甲、乙
    //
    if town_kana.starts_with("コウ、オツ") {
        // 7614103 カガワケン ショウズグントノショウチョウ コウ、オツ（オオキド）
        define!(PATTERN, r"^(コウ、オツ.*?)（.*）$");
        town_kana = PATTERN.replace_all(&town_kana, "$1").to_string();
        define!(DELIMITER, r"[、・]");
        let town_kanas: Vec<_> = DELIMITER.split(&town_kana).collect();
        return town_kanas.into_iter().map(Into::into).collect();
    }

    //
    // （Ａ～Ｂ丁目）
    //
    {
        // 2080032 トウキョウト ムサシムラヤマシ ミツギ（１－５チョウメ）
        // 1080023 トウキョウト ミナトク シバウラ（２－４チョウメ）
        define!(PATTERN, r"（([０-９]+)[～－]([０-９]+)チョウメ）");
        if let Some(caps) = PATTERN.captures(&town_kana) {
            let prefix = PATTERN.replace_all(&town_kana, "").to_string();
            let mut suffixes = zenkaku::zenkaku_range_label(
                format!("{}{}", &caps[1], "チョウメ").as_str(),
                format!("{}{}", &caps[2], "チョウメ").as_str(),
            );
            // 括弧有り丁目の場合は親住所も加える
            suffixes.push("".to_string());
            return suffixes.iter().map(|s| format!("{prefix}{s}")).collect();
        }
    }

    //
    // （第Ａ地割～第Ｂ地割）
    //
    {
        // 0285102 イワテケン イワテグンクズマキマチ クズマキ（ダイ４０チワリ＜５７バンチ１２５、１７６ヲノゾク＞－ダイ４５チワリ）
        define!(
            PATTERN,
            r"（(ダイ[０-９]+チワリ)[～－](ダイ[０-９]+チワリ)）"
        );
        if let Some(caps) = PATTERN.captures(&town_kana) {
            let prefix = PATTERN.replace_all(&town_kana, "").to_string();
            let suffixes = zenkaku::zenkaku_range_label(&caps[1], &caps[2]);
            // 括弧有り地割の場合は親住所は加えない
            return suffixes.iter().map(|s| format!("{prefix}{s}")).collect();
        }
    }

    //
    // 第Ａ地割～第Ｂ地割
    //
    {
        // 0287917 イワテケン クノヘグンヒロノチョウ タネイチダイ５０チワリ－ダイ７０チワリ（オオサワ、ジョウナイ、タキサワ）
        define!(PATTERN, r"(ダイ[０-９]+チワリ)[～－](ダイ[０-９]+チワリ)");
        if let Some(caps) = PATTERN.captures(&town_kana) {
            let prefix = PATTERN.replace_all(&town_kana, "").to_string();
            let mut suffixes = zenkaku::zenkaku_range_label(&caps[1], &caps[2]);
            // 括弧無し地割の場合は親住所も加える
            suffixes.push("".to_string());
            return suffixes.iter().map(|s| format!("{prefix}{s}")).collect();
        }
    }

    //
    // ＸＡ地割～ＸＢ地割
    //
    {
        // 0295523 イワテケン ワガグンニシワガマチ エッチュウハタ６４チワリ－エッチュウハタ６６チワリ
        define!(PATTERN, r"((.*?)[０-９]+チワリ)[～－](.*?[０-９]+チワリ)");
        if let Some(caps) = PATTERN.captures(&town_kana) {
            let prefix = PATTERN.replace_all(&town_kana, "").to_string();
            let mut suffixes = zenkaku::zenkaku_range_label(&caps[1], &caps[3]);
            // 括弧無し地割の場合は親住所も加える
            suffixes.push(caps[2].to_string());
            return suffixes.iter().map(|s| format!("{prefix}{s}")).collect();
        }
    }

    //
    // （Ａ・Ｂ・...）
    //
    {
        // 9401172 ニイガタケン ナガオカシ カマガシマ（ドテバタケ・フジバ）
        define!(PATTERN, r"（(.*?・.*?)）");
        if let Some(caps) = PATTERN.captures(&town_kana) {
            let prefix = PATTERN.replace_all(&town_kana, "").to_string();
            let inner = caps[1].to_string();
            let inner = inner.replace("、ソノタ", "");
            let inner = inner.replace("ヲフクム", "");
            let suffixes: Vec<_> = inner.split("・").collect();
            // 括弧有り中点列挙の場合は親住所を加えない
            return suffixes.iter().map(|s| format!("{prefix}{s}")).collect();
        }
    }

    //
    // （Ａ、Ｂ、...）or（Ａを含む）
    //
    {
        // 0580343 ホッカイドウ ホロイズミグンエリモチョウ トウヨウ（アブラコマ、ミナミトウヨウ、１３２－１５６、１５８－３５４、３６６、３６７バンチ）
        // 0790177 ホッカイドウ ビバイシ カミビバイチョウ（キョウワ、ミナミ）
        // 0482402 ホッカイドウ ヨイチグンニキチョウ オオエ（１チョウメ、２チョウメ＜６５１、６６２、６６８バンチ＞イガイ、３チョウメ５、１３－４、２０、６７８、６８７バンチ）
        // 4400845 アイチケン トヨハシシ タカシチョウ（キタハラ、ソノタ）
        // 7860301 コウチケン タカオカグンシマントチョウ タイショウ（ツヅラガワ、トドロキザキヲフクム）
        // 9960301 ヤマガタケン モガミグンオオクラムラ ミナミヤマ（４３０バンチイジョウ＜１７７０－１－２、１８６２－４２、１９２３－５ヲノゾク＞、オオヤチ、オリワタリ、カンカネノ、キンザン、タキノサワ、トヨマキ、ヌマノダイ、ヒジオリ、ヒラバヤシ）
        define!(PATTERN, r"（(.*?、.*?|.*?ヲフクム)）");
        if let Some(caps) = PATTERN.captures(&town_kana) {
            let prefix = PATTERN.replace_all(&town_kana, "").to_string();
            let inner = caps[1].to_string();
            let inner = inner.replace("、ソノタ", "");
            let inner = inner.replace("ヲフクム", "");
            let mut suffixes: Vec<_> = inner.split("、").collect();
            // 括弧有り句点列挙の場合は親住所も加える
            suffixes.push("");
            return suffixes.iter().map(|s| format!("{prefix}{s}")).collect();
        }
    }

    //
    // Ａ、Ｂ、...
    //
    {
        // 0295503 イワテケン ワガグンニシワガマチ アナアケ２２チワリ、アナアケ２３チワリ
        define!(PATTERN, r"(.*?、.*?)");
        if PATTERN.captures(&town_kana).is_some() {
            let town_kanas: Vec<_> = town_kana.split("、").collect();
            // 括弧無し句点列挙の場合は親住所を加えない
            return town_kanas.into_iter().map(Into::into).collect();
        }
    }

    //
    // 全ての処理が終わったはずなので、丸括弧と三角括弧を除去する
    // ただし（コウソウトウ）のような括弧は除去しない
    //
    {
        let skip_patterns = ["（コウソウトウ）"];
        define!(PATTERN1, r"（(.*?)）");
        town_kana = PATTERN1
            .replace_all(&town_kana, |caps: &regex::Captures| -> String {
                let matched = caps.get(0).unwrap().as_str();
                if skip_patterns.contains(&matched) {
                    matched.to_string()
                } else {
                    let inner = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();
                    inner.to_string()
                }
            })
            .to_string();
        define!(PATTERN2, r"＜(.*?)＞");
        town_kana = PATTERN2
            .replace_all(&town_kana, |caps: &regex::Captures| -> String {
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

    vec![town_kana]
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
            let (postal_code, prefecture_kana, city_kana, town_kana) =
                self.source.split(' ').collect_tuple().unwrap_or_else(|| {
                    panic!("Invalid source: {}", self.source);
                });
            UtfKenAllRecord {
                local_government_code: "00000".to_string(),
                old_postal_code: "000  ".to_string(),
                postal_code: postal_code.to_string(),
                prefecture_kana: prefecture_kana.to_string(),
                city_kana: city_kana.to_string(),
                town_kana: town_kana.to_string(),
                prefecture: "".to_string(),
                city: "".to_string(),
                town: "".to_string(),
                has_multi_postal_code: 0,
                has_chome: 0,
                has_multi_town: 0,
                update_code: 0,
                update_reason: 0,
            }
        }
    }

    #[datafile_test("./jp-postal-code-core/testdata/testcase_kana.yml")]
    fn test_normalize_utf_ken_all_record(testcase: TestCase) {
        let source = testcase.to_utf_ken_all_record();
        let mut result = normalize_utf_ken_all_record_town_kana(&source);
        let mut expect = testcase.result;
        result.sort();
        expect.sort();
        pretty_assertions::assert_eq!(result, expect);
    }
}
