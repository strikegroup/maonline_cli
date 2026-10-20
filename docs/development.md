# Development Guide

このドキュメントは、M&A Online CLI を開発・保守する人向けのガイドです。利用方法は [README](../README.md) を参照してください。

## 開発環境の準備

### 1. 前提ツールを用意する

- Rust（stable toolchain）
- Cargo

リリース設定を変更する場合は、`cargo-dist` 0.32.0 も必要です。

### 2. リポジトリを取得する

```bash
git clone https://github.com/strikegroup/maonline_cli
cd maonline_cli
```

### 3. API キーを登録する

M&A Online API を使う動作確認には API キーが必要です。`setup` で登録すると、OS 標準の設定ディレクトリにある `config.toml` へ保存されます。

```bash
cargo run -- setup --key <YOUR_MAONLINE_API_KEY>
cargo run -- status
```

検索コマンドの `--key` を指定した場合は、保存済みの API キーより優先されます。

### 4. 基本チェックを実行する

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
cargo run -- --help
cargo run -- companies --help
cargo run -- ma-data --help
cargo run -- news --help
cargo run -- news search --help
cargo run -- news get --help
cargo run -- setup --help
cargo run -- status --help
```

通常のテストは外部 API に接続しません。登録済み API キーを使う疎通確認は個別に実行します。

```bash
cargo run -- companies --name ストライク
cargo run -- ma-data --company ストライク
cargo run -- news search ストライク
```

## プロジェクト構成

このリポジトリは単一 crate 構成です。CLI と各 API のクライアントを同じ crate 内で管理します。

```text
.
├── .github/workflows/
├── docs/
│   └── development.md
├── src/
│   ├── app_config.rs
│   ├── app_paths.rs
│   ├── main.rs
│   ├── companies/
│   ├── ma_data/
│   └── news/
├── Cargo.toml
├── release-plz.toml
└── README.md
```

主な責務は次の通りです。

- `src/main.rs`: CLI のサブコマンド、ヘルプ、実行フローを定義します。
- `src/app_config.rs`: API キーの保存、読込、解決を担当します。
- `src/app_paths.rs`: OS 標準の設定・データ保存先を解決します。
- `src/companies/`: 企業情報の検索と出力モデルへの変換を担当します。
- `src/ma_data/`: M&A 案件の検索と出力モデルへの変換を担当します。
- `src/news/`: ニュースの検索、記事取得、出力モデルへの変換を担当します。

## API クライアントの構成

各機能は、CLI 引数と実行処理を持つ `command.rs`、HTTP 通信を行う `requester.rs`、API レスポンスを表す `raw_response.rs`、外部出力用モデルに分かれています。

API キーは Bearer 認証として送信します。企業情報、M&A 案件、ニュース検索は1ページ200件で取得し、`next_page` がある間は後続ページを結合します。出力は標準出力へ整形済み JSON として書き出し、エラーは `anyhow` のコンテキストを付けて呼び出し元へ返します。

API 項目やクエリパラメータを変更する場合は、次を合わせて確認してください。

- CLI 引数と API パラメータの対応
- URL エンコード
- ページネーション
- `raw_response` から出力モデルへの変換
- 欠損可能なフィールドの型
- README のコマンドリファレンスとレスポンス例

## テスト方針

単体テストでは、主に次を確認します。

- CLI ヘルプと引数の解析
- API クエリの組み立てとURLエンコード
- API レスポンスのデシリアライズ
- 外部出力モデルへの変換
- 記事IDなどの入力値検証

外部 API の可用性や実データに依存する確認は自動テストへ含めず、登録済み API キーを使って手動で確認します。API キーそのものをテストコード、ログ、コミットへ含めないでください。

## リリース

CI は push と pull request に対して、フォーマット、ビルド、テストを実行します。

`release-plz` がリリースPR、タグ、GitHub Releaseを管理し、リリース作成後に cargo-dist のワークフローを起動します。cargo-dist は各対象プラットフォームの配布物を作成し、GitHub Release の公開、Homebrew Formula の更新、npm パッケージの公開を行います。

`Cargo.toml` の `[workspace.metadata.dist]` を変更した場合は、ワークフローを再生成して差分を確認します。

```bash
dist generate
dist generate --check
dist plan
```

`.github/workflows/release.yml` と `strikegroup/homebrew-tap` の Formula は生成物です。通常は手作業で編集しません。

リリースには、GitHub Actions の次のSecretを使用します。

- `HOMEBREW_TAP_TOKEN`: Homebrew Tap の更新
- `CARGO_REGISTRY_TOKEN`: crates.io 公開を有効化した場合の認証

`release-plz.toml` では crates.io 公開を有効にしています。公開には `CARGO_REGISTRY_TOKEN` の登録が必要です。

## ライセンス更新

依存クレートのライセンス文書を更新するには、`cargo-about` 0.9.2 を使います。生成された `THIRD_PARTY_LICENSES.html` もリポジトリへコミットしてください。

```bash
cargo install --locked cargo-about --version 0.9.2 --features cli
cargo about generate --locked --fail --output-file THIRD_PARTY_LICENSES.html about.hbs
```
