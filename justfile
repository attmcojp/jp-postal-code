default:
  @just --list

# 全チェックコマンドを実行する
check:
  just lint-proto
  cargo clippy --all-targets --all-features

# 全テストコマンドを実行する
test:
  cargo test --all-targets --all-features && cargo test --doc

# 全フォーマットコマンドを実行する
fmt:
  cargo fmt --all
  just fmt-proto

# Protobuf からソースコードを生成する
gen-proto:
  rm -rf jp-postal-address/src/_gen
  cd proto && buf generate && buf build -o ../jp-postal-address/src/_gen/jp_postal_code.file_descriptor.binpb

# Protobuf の Lint を実行する
lint-proto:
  cd proto && buf lint

# Protobuf のフォーマットを実行する
fmt-proto:
  cd proto && buf format -w

# 開発サーバーを起動する
dev:
  cargo run --bin jp-postal-code

# Docker でサービスを起動する
up:
  docker compose up -d

# Docker でサービスを停止する
down:
  docker compose down

# Docker イメージをビルドする
build:
  docker compose build

# データベースマイグレーションを実行する
migrate:
  cargo run --bin jp-postal-code-update-database

# 必要なツールをインストールする（MacOS）
setup-tools-mac:
  brew install bufbuild/buf/buf
  brew install protobuf
  brew install just
