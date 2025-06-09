# jp-postal-code

郵便番号から住所を検索するシステム。HTTPサーバー、gRPCサーバー、CLIツールで構成されています。

## 構成

- **jp-postal-code**: 郵便番号検索のHTTP・gRPCサーバー
- **jp-postal-code-update-database**: 郵便番号データベース更新のCLIツール
- **jp-postal-code-core**: 郵便番号データの正規化処理
- **jp-postal-code-util**: 郵便番号データのダウンロード・パース処理
- **jp-postal-address**: gRPCサービス用のProtocol Buffers定義

## 起動

> [!NOTE]
>
> 初回起動時は自動的に郵便番号データベース構築を行なうため、少し起動に時間がかかります

```sh
docker compose build
docker compose up -d
```

HTTPサーバーはポート8000、gRPCサーバーはポート50051で起動します。

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

### 郵便番号検索（gRPC API）

gRPCサーバーもHTTP APIと同様の機能を提供します。ポート50051でサービスを提供しています。

Protocol Buffers定義: `proto/jp_postal_code/v1/`

#### grpcurl での例

```sh
# grpcurl のインストール（macOS）
brew install grpcurl

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

# ビルド済みバイナリを使用
./target/release/jp-postal-code-update-database --help
```

環境変数 `DATABASE_URL` でPostgreSQLの接続先を指定してください。

## 開発

### Just タスクランナー

開発効率化のため [just](https://github.com/casey/just) タスクランナーを使用しています。

#### インストール

```sh
# macOS
brew install just

# その他のプラットフォーム
# https://github.com/casey/just#installation を参照
```

#### 利用可能なタスク

```sh
# タスク一覧を表示
just

# 開発に必要なツールをインストール（macOS）
just setup-tools-mac

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

# Docker サービス起動
just up

# Docker サービス停止
just down

# Docker イメージビルド
just build

# データベースマイグレーション実行
just migrate
```

### 従来のCargo コマンド

```sh
# 全テスト実行
cargo test

# 特定のクレートのテスト実行
cargo test -p jp-postal-code-core

# 全クレートビルド
cargo build --release

# 特定のクレートビルド
cargo build -p jp-postal-code-update-database --release
```
