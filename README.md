# jp-postal-code

郵便番号から住所を検索するコンポーネント。

## Prepare

> [!NOTE]
>
> 初回起動時は自動的に郵便番号データベースん構築を行なうため、少し起動に時間がかかる

```sh
docker compose up -d
```

## Usage

cURL でも可能だが面倒なので [xh] で例を示す。

[xh]: https://github.com/ducaale/xh

### 郵便番号データベースの検索

郵便番号データベースから対応する住所を返す。以下のパラメータを指定可能

| パラメータ  | 説明                                                                          |
| ----------- | ----------------------------------------------------------------------------- |
| postal_code | 郵便番号（前方一致）                                                          |
| page_size   | 1ページあたりの件数（デフォルト: 10）                                         |
| page_token  | ページトークン。戻り値の `nextPageToken` を指定すると、その続きから結果を返す |

```
$ xh 'http://localhost:8000/api/search?postal_code=0120&page_size=3&page_token=eyJ1dGZfa2VuX2FsbF9pZCI6MTg0NTF9'
HTTP/1.1 200 OK
Content-Length: 324
Content-Type: application/json
Date: Wed, 16 Apr 2025 04:48:34 GMT

{
    "addresses": [
        {
            "postalCode": "0120013",
            "prefecture": "秋田県",
            "prefectureKana": "アキタケン",
            "city": "湯沢市",
            "cityKana": "ユザワシ",
            "town": "栄田",
            "townKana": "サカエダ"
        },
        {
            "postalCode": "0120035",
            "prefecture": "秋田県",
            "prefectureKana": "アキタケン",
            "city": "湯沢市",
            "cityKana": "ユザワシ",
            "town": "幸町",
            "townKana": "サイワイチョウ"
        },
        {
            "postalCode": "0120811",
            "prefecture": "秋田県",
            "prefectureKana": "アキタケン",
            "city": "湯沢市",
            "cityKana": "ユザワシ",
            "town": "桜通り",
            "townKana": "サクラドオリ"
        }
    ],
    "nextPageToken": "eyJ1dGZfa2VuX2FsbF9pZCI6MTg0NTV9"
}
```

### 郵便番号データベースの更新

郵便局が配布している `ken_all_utf8.zip` をダウンロードして、郵便番号データベースを更新する。

https://www.post.japanpost.jp/zipcode/download.html

```
$ xh POST http://localhost:8000/api/update                                                                      imp :^ ~/ogh/attmcojp/jp-postal-code
HTTP/1.1 204 No Content
Date: Wed, 16 Apr 2025 04:49:12 GMT
```
