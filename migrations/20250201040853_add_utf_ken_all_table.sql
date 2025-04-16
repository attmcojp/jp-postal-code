drop table if exists utf_ken_all cascade;
create table utf_ken_all (
  -- レコードのID
  utf_ken_all_id bigint generated always as identity primary key,
  -- 全国地方公共団体コード（JIS X0401、X0402）
  local_government_code varchar(5) not null,
  -- （旧）郵便番号（5桁）
  old_postal_code varchar(5) not null,
  -- 郵便番号（7桁）
  postal_code varchar(7) not null,
  -- 都道府県名（仮名）
  prefecture_kana varchar(20) not null,
  -- 市区町村名（仮名）
  city_kana varchar(50) not null,
  -- 町域名（仮名）
  town_kana varchar(1000) not null,
  -- 都道府県名
  prefecture varchar(20) not null,
  -- 市区町村名
  city varchar(50) not null,
  -- 町域名
  town varchar(1000) not null,
  -- 一町域が二以上の郵便番号で表される場合の表示（「1」は該当、「0」は該当せず）
  has_multi_postal_code smallint not null,
  -- 一郵便番号が二以上の町域で表される場合の表示（「1」は該当、「0」は該当せず）
  has_chome smallint not null,
  -- 一町域が二以上の市区町村で表される場合の表示（「1」は該当、「0」は該当せず）
  has_multi_town smallint not null,
  -- 更新の表示（「0」は変更なし、「1」は変更あり、「2」廃止（廃止データのみ使用））
  update_code smallint not null,
  -- 変更理由
  -- 0: 変更なし
  -- 1: 市政・区政・町政・分区・政令指定都市施行
  -- 2: 住居表示の実施
  -- 3: 区画整理
  -- 4: 郵便区調整等
  -- 5: 訂正
  -- 6: 廃止（廃止データのみ使用）
  update_reason smallint not null,
  -- レコードの更新日時
  updated_at timestamp with time zone not null
);
