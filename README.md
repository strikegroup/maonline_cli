# M&A Online CLI

[M&A Online](https://maonline.jp/) から、企業情報、M&A案件、ニュース・速報をコマンドラインから検索・取得するツールです。取得結果は JSON 形式で標準出力に書き出します。

## できること

- 証券コード、EDINETコード、企業名などによる企業情報の検索
- 当事者、買収金額、スキームなどによるM&A案件の検索
- ニュース、大量保有速報、TOB速報の検索と記事取得

## インストール

```bash
cargo install --git https://github.com/strikegroup/maonline_cli
```

## セットアップ

M&A Online の管理者から API キーを取得し、最初に登録します。

```bash
maonline setup --key YOUR_API_KEY
```

登録後は、各検索コマンドで API キーを指定する必要はありません。登録状態は `maonline status` で確認できます。

## ユースケース

以下のレスポンス例は、主要なフィールドと検索結果を抜粋しています。

### 企業名から企業情報を調べる

企業名またはカナの一部を指定して、該当する企業の証券コード、EDINETコード、市場区分、所在地などを取得します。

```bash
maonline companies --name ストライク
```

```json
[
  {
    "sec_code": "6196",
    "edinet_code": "E32380",
    "company_name": "ストライクグループ",
    "market": "東証プライム",
    "address": "千代田区大手町一丁目２番１号",
    "homepage": "http://www.strike.co.jp/"
  },
  {
    "sec_code": "9816",
    "edinet_code": "E02738",
    "company_name": "ストライダーズ",
    "market": "東証スタンダード",
    "address": "東京都港区新橋５－１３－５ 新橋ＭＣＶビル",
    "homepage": "http://www.striders.co.jp/"
  }
]
```

### 証券コードから企業情報を取得する

証券コードを指定すると、完全一致する企業を検索できます。

```bash
maonline companies --sec-code 7203
```

```json
[
  {
    "sec_code": "7203",
    "edinet_code": "E02144",
    "company_name": "トヨタ自動車",
    "market": "東証プライム、名証プレミア",
    "address": "愛知県豊田市トヨタ町１",
    "homepage": "http://www.toyota.co.jp/"
  }
]
```

### 楽天が関係するM&A案件を調べる

買い手、対象、売り手のいずれかに指定した企業名を含むM&A案件を検索します。

```bash
maonline ma-data --company 楽天
```

```json
[
  {
    "disdate": "2018-01-29",
    "title": "楽天＜4755＞、野村HD＜8604＞傘下の朝日火災をTOBで買収へ",
    "price": 44998391232,
    "total": 44998391232,
    "maoscheme": "4",
    "maoschemename": "株式譲渡",
    "maoschemesub": "1",
    "maoschemesubname": "ＴＯＢ",
    "inout": "IN-IN",
    "buyer_info": {
      "name": "楽天(株)",
      "code": "4755",
      "industry": "IT・ソフトウエア",
      "area": "東京都"
    },
    "target_info": {
      "name": "朝日火災海上保険(株)",
      "code": "",
      "industry": "証券・商品先物",
      "area": "東京都"
    },
    "seller_info": {
      "name": "野村ホールディングス(株)",
      "code": "8604",
      "industry": "証券・商品先物",
      "area": "東京都"
    }
  }
]
```

### 1兆円以上の規模のM&A案件を抽出する

買収金額の下限を指定して、大型案件を絞り込みます。

```bash
maonline ma-data --min-price 1000000000000
```

```json
[
  {
    "disdate": "2025-06-03",
    "title": "トヨタグループ、豊田自動織機＜6201＞をTOBで非公開化",
    "price": 5920924771888,
    "total": 5920924771888,
    "maoscheme": "4",
    "maoschemename": "株式譲渡",
    "maoschemesub": "1",
    "maoschemesubname": "ＴＯＢ",
    "inout": "IN-IN",
    "buyer_info": {
      "name": "トヨタ不動産(株)",
      "code": "",
      "industry": "住宅・不動産",
      "area": "東京都"
    },
    "target_info": {
      "name": "(株)豊田自動織機",
      "code": "6201",
      "industry": "輸送用機器",
      "area": "愛知県"
    },
    "seller_info": {
      "name": "株主",
      "code": "",
      "industry": "",
      "area": ""
    }
  }
]
```

### ストライクに関するニュースを検索

タイトル、本文、タグを対象にニュース記事を検索します。

```bash
maonline news search ストライク
```

```json
[
  {
    "id": "20230113jp",
    "title": "M＆A総合研究所、持ち株会社制移行で3月に「M＆A総研ホールディングス」に社名変更",
    "body_html": "<p>記事本文...</p>",
    "published_at": "2023-01-13T16:02:00+09:00",
    "related_codes": [
      "9552"
    ]
  }
]
```

### TOB速報を絞り込む

記事種別を `tob_news` に指定して、TOB速報だけを検索します。

```bash
maonline news search トヨタ --article-type tob_news
```

```json
[
  {
    "id": "2026_004a",
    "title": "トヨタアセット準備（トヨタ不動産）、豊田自動織機＜6201＞にTOB開始",
    "body_html": "<p>記事本文...</p>",
    "published_at": "2026-01-15T00:00:00+09:00",
    "related_codes": [
      "6201",
      "E01514",
      "E41388"
    ]
  }
]
```

### 記事IDからニュースを取得する

検索結果に含まれる記事IDを指定して、記事を1件取得します。 (記事IDが分かっているときのみ)

```bash
maonline news get summary20251011
```

```json
{
  "id": "summary20251011",
  "title": "1週間のM&A速報まとめ（2025年10月6日〜2025年10月10日）",
  "body_html": "<h3 class='dateheader'>2025年10月6日</h3><p>記事本文...</p>"
}
```

## コマンドリファレンス

### コマンド一覧

| コマンド | 短縮形 | 概要 |
|---|---|---|
| `companies` | `c` | 企業情報を検索します |
| `ma-data` | `m` | M&A案件を検索します |
| `news` | `n` | ニュース・速報を検索・取得します |
| `setup` | － | API キーを登録します |
| `status` | `s` | 設定状態を確認します |

各コマンドの詳細はヘルプでも確認できます。`-h` と `--help` の出力内容は同じです。

```bash
maonline --help
maonline companies --help
maonline ma-data --help
maonline news search --help
```

### `companies`

証券コード、企業名、EDINETコード、市場区分のうち、1つ以上を指定します。複数指定した場合は、すべての条件で絞り込みます。

| オプション | 内容 |
|---|---|
| `-s, --sec-code SEC_CODE` | 証券コードで完全一致検索 |
| `-n, --name COMPANY_NAME` | 企業名またはカナで曖昧検索 |
| `-e, --edinet-code EDINET_CODE` | EDINETコードで完全一致検索 |
| `-m, --market MARKET` | 市場区分で絞り込み |
| `-k, --key API_KEY` | API キーを明示的に指定する場合 |

### `ma-data`

企業名、買収金額、スキーム、国内外区分、エリアを組み合わせてM&A案件を絞り込みます。

| オプション | 内容 |
|---|---|
| `-c, --company COMPANY_NAME` | 買い手・対象・売り手の企業名で曖昧検索 |
| `--min-price YEN` | 買収金額の下限を円単位で指定 |
| `--max-price YEN` | 買収金額の上限を円単位で指定 |
| `-s, --scheme CODE` | M&Aスキームコードで絞り込み |
| `-i, --inout CATEGORY` | 国内外区分で絞り込み |
| `-a, --area AREA` | 対象エリア名で絞り込み |
| `-k, --key API_KEY` | API キーを明示的に指定する場合 |

### `news search`（短縮形: `news s`）

キーワード、関連企業、記事種別、公開日の範囲を組み合わせてニュース記事を検索します。条件を省略した場合は全記事が対象です。

API キーを明示的に指定する場合は、`news` の直後、サブコマンドの前に指定します。

```text
maonline news --key API_KEY search QUERY
```

| 引数・オプション | 内容 |
|---|---|
| `QUERY` | タイトル、本文、タグを対象にキーワード検索 |
| `-c, --company COMPANY_NAME` | 関連企業名で絞り込み |
| `-a, --article-type TYPE` | 記事種別で絞り込み |
| `-f, --date-from YYYY-MM-DD` | 指定日以降に公開された記事へ絞り込み |
| `-t, --date-to YYYY-MM-DD` | 指定日以前に公開された記事へ絞り込み |
| `-w, --with-raw` | 解析前の本文を `body_raw` として出力 |

記事種別には次の値を指定できます。

| 値 | 記事種別 |
|---|---|
| `news` | ニュース |
| `shareholdings` | 大量保有速報 |
| `tob_news` | TOB速報 |

### `news get`（短縮形: `news g`）

```text
maonline news get ARTICLE_ID
```

検索結果に含まれる記事IDを指定して、ニュース記事を1件取得します。

### `setup`

```text
maonline setup --key API_KEY
```

M&A Online API キーを登録します。

### `status`

```text
maonline status
```

API キーの登録状態、設定ファイルのパス、データディレクトリを表示します。API キーそのものは表示しません。

## ライセンス

このソフトウェアは [Apache License 2.0](LICENSE) で提供します。

依存クレートのライセンス文書を更新するには、`cargo-about` 0.9.2をインストールして生成コマンドを実行します。生成された`THIRD_PARTY_LICENSES.html`もリポジトリへコミットしてください。

```bash
cargo install --locked cargo-about --version 0.9.2
cargo about generate --locked --fail --output-file THIRD_PARTY_LICENSES.html about.hbs
```
