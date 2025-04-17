create table utf_ken_all (
  utf_ken_all_id bigint generated always as identity primary key,
  local_government_code text not null,
  old_postal_code varchar(5) not null,
  postal_code varchar(7) not null,
  prefecture_kana text not null,
  city_kana text not null,
  town_kana text not null,
  prefecture text not null,
  city text not null,
  town text not null,
  has_multi_postal_code smallint not null,
  has_chome smallint not null,
  has_multi_town smallint not null,
  update_code smallint not null,
  update_reason smallint not null,
  updated_at timestamp with time zone not null
);

create index idx_utf_ken_all_postal_code_town_town_kana on utf_ken_all (postal_code, town, town_kana);

comment on table utf_ken_all is '郵便局が配布している郵便番号データを正規化したデータ';
comment on column utf_ken_all.utf_ken_all_id is 'レコードのID';
comment on column utf_ken_all.local_government_code is '全国地方公共団体コード（JIS X0401、X0402）';
comment on column utf_ken_all.old_postal_code is '（旧）郵便番号（5桁）';
comment on column utf_ken_all.postal_code is '郵便番号（7桁）';
comment on column utf_ken_all.prefecture_kana is '都道府県名（仮名）';
comment on column utf_ken_all.city_kana is '市区町村名（仮名）';
comment on column utf_ken_all.town_kana is '町域名（仮名）';
comment on column utf_ken_all.prefecture is '都道府県名';
comment on column utf_ken_all.city is '市区町村名';
comment on column utf_ken_all.town is '町域名';
comment on column utf_ken_all.has_multi_postal_code is '一町域が二以上の郵便番号で表される場合の表示（「1」は該当、「0」は該当せず）';
comment on column utf_ken_all.has_chome is '一郵便番号が二以上の町域で表される場合の表示（「1」は該当、「0」は該当せず）';
comment on column utf_ken_all.has_multi_town is '一町域が二以上の市区町村で表される場合の表示（「1」は該当、「0」は該当せず）';
comment on column utf_ken_all.update_code is '更新の表示（「0」は変更なし、「1」は変更あり、「2」廃止（廃止データのみ使用））';
comment on column utf_ken_all.update_reason is '変更理由';
comment on column utf_ken_all.updated_at is 'レコードの更新日時';
