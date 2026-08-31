# Research API V1 エンドポイント詳細仕様書

## 1. 共通仕様

- **Base URL:** `https://[your-domain]/research/api/v1`
    
- **認証方式:** HTTP Bearer認証
    
    - Header: `Authorization: Bearer <API_TOKEN>`
        
- **データ形式:** JSON (`Content-Type: application/json`)
    
- **共通リクエストパラメータ:**
    
    |**パラメータ名**|**型**|**既定値**|**内容**|
    |---|---|---|---|
    |`page`|Integer|1|取得したいページ番号|
    |`per_page`|Integer|50|1ページあたりの件数 (**最大 200**)|
    

---

## 2. エンドポイント

### 2.1 企業検索 API (`GET /companies`)

企業の基本情報を検索・取得します。

#### リクエストパラメータ

|**パラメータ名**|**型**|**必須**|**内容**|
|---|---|---|---|
|`syokencode`|String|任意|証券コードでの完全一致検索|
|`edcode`|String|任意|EDINETコードでの完全一致検索|
|`name`|String|任意|企業名またはカナでの曖昧検索（法人格は自動除外）|
|`market`|String|任意|市場区分での絞り込み|

#### レスポンス形式

JSON

```
{
  "data": [
    {
      "edsyokencode": "1234",
      "edcode": "E01234",
      "edname": "株式会社サンプル",
      "market": "東証プライム",
      "address": "東京都千代田区...",
      "homepage": "https://example.com"
    }
  ],
  "meta": {
    "current_page": 1,
    "next_page": 2,
    "prev_page": null,
    "total_pages": 5,
    "total_count": 230
  }
}
```

---

### 2.2 M&Aデータ検索 API (`GET /ma_data`)

M&A案件の統計・詳細データを検索します。

#### リクエストパラメータ

|**パラメータ名**|**型**|**必須**|**内容**|
|---|---|---|---|
|`company_name`|String|任意|当事者（買い手・対象・売り手）の企業名曖昧検索|
|`min_price`|Integer|任意|買収金額の下限（円単位）|
|`max_price`|Integer|任意|買収金額の上限（円単位）|
|`scheme`|String|任意|M&Aスキームコード|
|`inout`|String|任意|IN-IN / IN-OUT 等の区分|
|`area`|String|任意|対象エリア名|

#### レスポンス形式

JSON

```
{
  "data": [
    {
      "disdate": "2024-05-20",
      "title": "株式会社Aによる株式会社Bの買収",
      "price": 500000000,
      "maoschemename": "株式譲渡",
      "buyer_info": { "name": "株式会社A", "code": "1111", "industry": "サービス業", "area": "東京都" },
      "target_info": { "name": "株式会社B", "code": "2222", "industry": "製造業", "area": "大阪府" }
    }
  ],
  "meta": {
    "current_page": 1,
    "total_pages": 12,
    "total_count": 580
  }
}
```

---

### 2.3 ニュース・速報一覧 API (`GET /news`)

ニュース、大量保有報告、TOB速報の一覧を複合的に検索します。

#### リクエストパラメータ

|**パラメータ名**|**型**|**必須**|**内容**|
|---|---|---|---|
|`q`|String|任意|キーワード検索（タイトル、本文、タグ対象）|
|`company_name`|String|任意|関連企業名での検索|
|`article_type`|String|任意|`news`, `shareholdings`, `tob_news`|
|`date_from`|Date|任意|公開日開始 (YYYY-MM-DD)|
|`date_to`|Date|任意|公開日終了 (YYYY-MM-DD)|
|`with_raw`|String|任意|`"true"` で解析前の元データ (`body_raw`) を付与|

#### レスポンス形式

JSON

```
{
  "data": [
    {
      "id": "news/20240520_sample",
      "title": "大量保有報告書の提出：株式会社サンプル",
      "body_html": "<p>本文...</p>",
      "published_at": "2024-05-20T10:00:00+09:00",
      "related_codes": ["1234"]
    }
  ],
  "meta": { "current_page": 1, "total_count": 1200 }
}
```

---

### 2.4 ニュース詳細 API (`GET /news/:id`)

特定の記事の詳細情報を取得します。一覧で取得した `id` (URL形式) を使用します。

#### レスポンス形式

JSON

```
{
  "id": "news/20240520_sample",
  "title": "大量保有報告書の提出：株式会社サンプル",
  "body_html": "<div>詳細なHTML本文...</div>",
  "published_at": "2024-05-20T10:00:00+09:00",
  "related_codes": ["1234"],
  "body_raw": "{ \"original_json\": ... }"
}
```

---

## 3. レスポンスステータス

|**ステータスコード**|**意味**|**内容**|
|---|---|---|
|**200 OK**|成功|正常に処理されました。|
|**401 Unauthorized**|認証エラー|有効なAPIキーが設定されていません。|
|**404 Not Found**|未検出|指定されたIDのリソースが見つかりません。|
|**422 Unprocessable Entity**|パラメータ不正|リクエスト内容に不備があります。|
|**500 Server Error**|サーバーエラー|システム内部でエラーが発生しました。|

---

**運用メモ:** ページネーションにより、大量のデータを取得する際は `page` をインクリメントして連続リクエストを行うことができます。1回あたりの最大取得数は **200件** です。
