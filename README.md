# M&A Online CLI

[M&A Online](https://maonline.jp/) から、M&A案件をコマンドラインから検索するツールです。取得結果は JSON 形式で標準出力に書き出します。

## できること

- 当事者、買収金額、スキームなどによるM&A案件の検索

## インストール

### Cargo

```bash
cargo install --locked maonline_cli
```

### Homebrew

```bash
brew install strikegroup/tap/maonline
```

### npm

```bash
npm install --global maonline-cli
```

### APT（Ubuntu / Debian）

```bash
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://strikegroup.github.io/maonline_cli/apt/maonline-cli-archive-keyring.gpg \
  | sudo tee /etc/apt/keyrings/maonline-cli-archive-keyring.gpg > /dev/null
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/maonline-cli-archive-keyring.gpg] https://strikegroup.github.io/maonline_cli/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/maonline-cli.list > /dev/null
sudo apt update && sudo apt install maonline-cli
```

## 利用規約

[M＆A Online Research Kit利用特約](https://maonline.jp/pages/policy_researchkit) に従って本ツールを利用してください。

## セットアップ

M&A Online の管理者から API キーを取得し、最初に登録します。

```bash
maonline setup --key YOUR_API_KEY
```

環境変数で指定することもできます。

```bash
export MAONLINE_API_KEY=YOUR_API_KEY
```

検索時は `--key` → `MAONLINE_API_KEY` → 保存済み設定の順で使用します。空の環境変数は無視します。作業ディレクトリの `.env` も読み込みます。`status` は保存済み設定の登録状態を表示します。


通信状況を調べる場合は `-v`、ログを減らす場合は `-q` を指定します。診断ログは標準エラーへ、検索結果のJSONは標準出力へ出力します。

```bash
maonline -v ma-data --company ストライク > ma_data.json
```

## ユースケース

以下のレスポンス例は、主要なフィールドと検索結果を抜粋しています。

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

## コマンドリファレンス

### コマンド一覧

| コマンド | 短縮形 | 概要 |
|---|---|---|
| `ma-data` | `m` | M&A案件を検索します |
| `setup` | － | API キーを登録します |
| `status` | `s` | 設定状態を確認します |

各コマンドの詳細はヘルプでも確認できます。`-h` と `--help` の出力内容は同じです。

```bash
maonline --help
maonline ma-data --help
```

### 検索結果の件数・ページ指定

`ma-data` では、次のオプションを使えます。

| オプション | 内容 |
|---|---|
| `-l, --limit COUNT` | 1ページの取得件数（1〜200）。未指定または `0` は全件取得 |
| `-p, --page PAGE` | 取得するページ番号（1以上、既定は1） |

`--limit` が1以上の場合は指定ページだけを取得し、後続ページは取得しません。`--limit` が未指定または `0` の場合、`--page` は1のみ指定できます。出力は従来どおりJSON配列です。

```bash
maonline ma-data --company 楽天 --limit 20 --page 2
```

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

買収金額は下限が上限以下になるよう指定します。

### `setup`

```text
maonline setup --key API_KEY
```

利用特約のURLと同意確認を表示し、同意すると M&A Online API キーを登録します。再登録時も同意確認を行います。

### `status`

```text
maonline status
```

API キーの登録状態、設定ファイルのパス、データディレクトリを表示します。API キーそのものは表示しません。

## 開発者向け情報

開発環境の準備、実装構成、テスト、リリース手順は [Development Guide](docs/development.md) を参照してください。

## ライセンス

このソフトウェアは [Apache License 2.0](LICENSE) で提供します。
