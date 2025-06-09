# jp-postal-code

郵便番号から住所を検索するシステム。HTTPサーバーとCLIツールで構成されています。

## 構成

- **jp-postal-code**: 郵便番号検索のHTTPサーバー
- **jp-postal-code-update-database**: 郵便番号データベース更新のCLIツール
- **jp-postal-code-core**: 郵便番号データの正規化処理
- **jp-postal-code-util**: 郵便番号データのダウンロード・パース処理

## 起動

> [!NOTE]
>
> 初回起動時は自動的に郵便番号データベース構築を行なうため、少し起動に時間がかかります

```sh
docker compose build
docker compose up -d
```

HTTPサーバーはポート8000で起動します。

## 使用方法

### 郵便番号検索（HTTP API）

郵便番号データベースから対応する住所を返します。以下のパラメータを指定可能：

| パラメータ  | 説明                                                                          |
| ----------- | ----------------------------------------------------------------------------- |
| postal_code | 郵便番号（前方一致）                                                          |
| page_size   | 1ページあたりの件数（デフォルト: 10）                                         |
| page_token  | ページトークン。戻り値の `nextPageToken` を指定すると、その続きから結果を返す |

```sh
# cURLの例
curl 'http://localhost:8000/api/search?postal_code=0120&page_size=3'

# xhの例
xh 'http://localhost:8000/api/search?postal_code=0120&page_size=3&page_token=eyJ1dGZfa2VuX2FsbF9pZCI6MTg0NTF9'
```

レスポンス例：
```json
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
        }
    ],
    "nextPageToken": "eyJ1dGZfa2VuX2FsbF9pZCI6MTg0NTV9"
}
```

### 郵便番号データベースの更新（CLI）

郵便局が配布している `ken_all_utf8.zip` をダウンロードして、郵便番号データベースを更新します。

データソース: https://www.post.japanpost.jp/zipcode/download.html

```sh
# デフォルトのURL（日本郵便）からダウンロード・更新
cargo run -p jp-postal-code-update-database

# カスタムURLから更新
cargo run -p jp-postal-code-update-database -- --url "https://example.com/ken_all_utf8.zip"

# ビルド済みバイナリを使用
./target/release/jp-postal-code-update-database --help
```

環境変数 `DATABASE_URL` でPostgreSQLの接続先を指定してください。

## 開発

### テスト実行

```sh
# 全テスト実行
cargo test

# 特定のクレートのテスト実行
cargo test -p jp-postal-code-core
```

### ビルド

```sh
# 全クレートビルド
cargo build --release

# 特定のクレートビルド
cargo build -p jp-postal-code-update-database --release
```
