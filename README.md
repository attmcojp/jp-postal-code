# jp-postal-code

郵便番号から住所を検索するコンポーネント。

## Prepare

```sh
docker compose build
docker compose up -d
```

## Usage

### 郵便番号データベースの更新

```
curl -X POST 'http://localhost:8000/update'
```

### 郵便番号データベースの検索

```
curl 'http://localhost:8000/search?postal_code=123'
```
