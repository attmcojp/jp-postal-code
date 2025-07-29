# jp-postal-code

郵便番号から住所を検索するシステム。HTTPサーバー、gRPCサーバー、CLIツールで構成されています。

## 構成

- **jp-postal-code**: 郵便番号検索のHTTP・gRPCサーバー
- **jp-postal-code-update-database**: 郵便番号データベース更新のCLIツール
- **jp-postal-code-core**: 郵便番号データの正規化処理
- **jp-postal-code-util**: 郵便番号データのダウンロード・パース処理
- **jp-postal-code-proto**: gRPCサービス用のProtocol Buffers定義

## 起動

> [!NOTE]
>
> 初回起動時は自動的に郵便番号データベース構築を行なうため、少し起動に時間がかかります

```sh
cp .env.sample .env
docker compose up -d
```

HTTPサーバーはポート8000、gRPCサーバーはポート50051で起動します。

## 環境変数

アプリケーションは以下の環境変数で設定をカスタマイズできます：

| 環境変数           | 説明                            | デフォルト値                                             | 必須 |
| ------------------ | ------------------------------- | -------------------------------------------------------- | ---- |
| `DATABASE_URL`     | PostgreSQLデータベースの接続URL | -                                                        | ✓    |
| `HTTP_SERVER_ADDR` | HTTPサーバーのリッスンアドレス  | `localhost:8000` (開発環境)<br>`0.0.0.0:80` (Docker)     | -    |
| `GRPC_SERVER_ADDR` | gRPCサーバーのリッスンアドレス  | `localhost:50051` (開発環境)<br>`0.0.0.0:50051` (Docker) | -    |

## 使用方法

郵便番号データベースから対応する住所を返します。以下のパラメータを指定可能：

| パラメータ  | 説明                                                                          |
| ----------- | ----------------------------------------------------------------------------- |
| postal_code | 郵便番号（前方一致）                                                          |
| page_size   | 1ページあたりの件数（デフォルト: 10）                                         |
| page_token  | ページトークン。戻り値の `nextPageToken` を指定すると、その続きから結果を返す |

### REST API

```sh
# 郵便番号検索
curl 'http://localhost:8000/api/search?postal_code=0120&page_size=3'
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

### gRPC

```sh
# サービス一覧の確認
grpcurl -plaintext localhost:50051 list

# メソッド一覧の確認
grpcurl -plaintext localhost:50051 list jp_postal_code.v1.PostalAddressService

# 郵便番号検索
grpcurl -plaintext -d '{
  "postal_code": "0120",
  "page_size": 3
}' localhost:50051 jp_postal_code.v1.PostalAddressService/SearchPostalAddress
```

レスポンス例：

```json
{
  "items": [
    {
      "address": {
        "postalCode": "0120013",
        "prefecture": "秋田県",
        "prefectureKana": "アキタケン",
        "city": "湯沢市",
        "cityKana": "ユザワシ",
        "town": "栄田",
        "townKana": "サカエダ"
      }
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

# Dockerイメージを使用
docker compose run --rm -it jp-postal-code /bin/update-database
```

## 開発

### Just タスクランナー

開発効率化のため [just](https://github.com/casey/just) タスクランナーを使用しています。

```sh
# タスク一覧を表示
just

# コードの品質チェック（clippy + protobuf lint）
just check

# テスト実行
just test

# コードフォーマット（Rust + protobuf）
just fmt

# Protocol Buffers からRustコードを生成
just gen-proto

# 開発サーバー起動
just dev

# データベースマイグレーション実行
just migrate
```
