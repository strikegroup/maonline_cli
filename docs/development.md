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

検索時の API キーは `--key` → 環境変数 `MAONLINE_API_KEY` → `config.toml` の順で解決します。空の環境変数は無視します。作業ディレクトリの `.env` も読み込みます。必要なら `.env.example` をコピーして設定してください（`.env` はGit管理対象外です）。`setup` は明示的なキー指定と利用特約への同意を必要とします。

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
│   ├── api_client.rs
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
- `src/api_client.rs`: 共通HTTPクライアント、タイムアウト、再試行、ページ取得を担当します。
- `src/app_config.rs`: API キーの保存、読込、解決を担当します。
- `src/app_paths.rs`: OS 標準の設定・データ保存先を解決します。
- `src/companies/`: 企業情報の検索と出力モデルへの変換を担当します。
- `src/ma_data/`: M&A 案件の検索と出力モデルへの変換を担当します。
- `src/news/`: ニュースの検索、記事取得、出力モデルへの変換を担当します。

## API クライアントの構成

各機能は、CLI 引数と実行処理を持つ `command.rs`、HTTP 通信を行う `requester.rs`、API レスポンスを表す `raw_response.rs`、外部出力用モデルに分かれています。

HTTP 通信は `api_client.rs` に集約しています。接続タイムアウトは5秒、1リクエストは30秒です。通信失敗と HTTP 429・500・502・503・504 は最大3回再試行します。待機は指数バックオフと `Retry-After`（秒数またはHTTP日時）に対応し、1回あたり最大30秒です。認証失敗や不正なJSONは再試行せず、エラーを分類して返します。

API キーは Bearer 認証として送信します。企業情報、M&A 案件、ニュース検索は1ページ200件で取得し、`next_page` がある間は後続ページを結合します。出力は標準出力へ整形済み JSON として書き出し、エラーは `anyhow` のコンテキストを付けて呼び出し元へ返します。

API 項目やクエリパラメータを変更する場合は、次を合わせて確認してください。

- CLI 引数と API パラメータの対応
- URL エンコード
- ページネーション
- `raw_response` から出力モデルへの変換
- 欠損可能なフィールドの型
- README のコマンドリファレンスとレスポンス例

## 診断ログ

`tracing` でアプリケーションの診断を標準エラーへ出力します。既定はinfo、`-v` はdebug、`-vv` はtrace、`-q` はwarn、`-qq` はerror、`-qqq` は非表示です。フラグはサブコマンドの前後で使用できます。

コマンド名、試行回数、HTTPステータス、ページ番号、再試行待機時間を記録します。キー、検索条件、URL、レスポンス本文、引数全体は記録しません。依存ライブラリのログと `RUST_LOG` は有効にしません。エラーへ元のHTTPエラーやTOML解析エラーを付けると機密情報が混入するため、分類済みのエラーに変換してください。

## テスト方針

単体テストでは、主に次を確認します。

- CLI ヘルプと引数の解析
- API クエリの組み立てとURLエンコード
- API レスポンスのデシリアライズ
- 外部出力モデルへの変換
- ローカル模擬HTTPサーバー（wiremock）での認証ヘッダー、ページ取得、429・5xxの再試行、401・不正JSON・タイムアウト
- 記事IDなどの入力値検証

外部 API の可用性や実データに依存する確認は自動テストへ含めず、登録済み API キーを使って手動で確認します。API キーそのものをテストコード、ログ、コミットへ含めないでください。

## リリース

CI は push と pull request に対して、フォーマット、ビルド、テストを実行します。

リリースは release-plz が作成する Release PR のマージを契機に行います。`release_always = false` により、通常の変更を `main` に push しただけでは公開しません。手動のタグ作成や先行した `cargo publish` は避けてください。

初回は未公開 crate の更新差分がないため、配布設定のマージと認証設定を済ませた後、`Release-plz` ワークフローを `main` に対して手動実行し、`initial_release` を `true` にします。この操作で初回の crates.io 公開とバイナリ配布が開始されます。以降の公開は Release PR を使用します。

`release-plz` がリリースPR、タグ、GitHub Releaseを管理し、リリース作成後に cargo-dist のワークフローを起動します。cargo-dist は各対象プラットフォームの配布物を作成し、GitHub Release の公開、Homebrew Formula の更新、npm パッケージの公開を行います。リリース公開後に、APT ワークフローが署名付きリポジトリを GitHub Pages へ公開します。

`Cargo.toml` の `[workspace.metadata.dist]` を変更した場合は、ワークフローを再生成して差分を確認します。

```bash
dist generate
dist generate --check
dist plan
```

`.github/workflows/release.yml` と `strikegroup/homebrew-tap` の Formula は生成物です。通常は手作業で編集しません。

リリースには、GitHub Actions の次のSecretを使用します。

- `HOMEBREW_TAP_TOKEN`: Homebrew Tap の更新
- `CARGO_REGISTRY_TOKEN`: crates.io 公開の認証
- `APT_GPG_PRIVATE_KEY`: APT リポジトリの署名用秘密鍵
- `APT_GPG_PASSPHRASE`: APT 署名鍵のパスフレーズ

`release-plz.toml` では crates.io 公開を有効にしています。公開には `CARGO_REGISTRY_TOKEN` の登録が必要です。

npm は初回リリースのバイナリ公開後にパッケージを手動公開し、Trusted Publisher に `strikegroup/maonline_cli` の `release.yml` を登録します。設定完了後、GitHub Actions の Repository variable `NPM_TRUSTED_PUBLISHING_ENABLED` を `true` にすると、以降のリリースで npm の自動公開が有効になります。未設定時は npm の公開ジョブをスキップし、他の配布処理を継続します。`NPM_TOKEN` は使用しません。

APT の公開には GitHub Pages の Source を GitHub Actions に設定する必要があります。

APT ジョブがスキップされた場合など、公開済みの GitHub Release から APT だけを再配布するには、`main` に対して次を実行します。指定したリリースの Linux 成果物とチェックサムを取得し、検証後に deb の生成・署名と Pages への配置を行います。

```bash
gh workflow run publish-apt.yml --ref main -f tag=v0.0.1
```

## ライセンス更新

依存クレートのライセンス文書を更新するには、`cargo-about` 0.9.2 を使います。生成された `THIRD_PARTY_LICENSES.html` もリポジトリへコミットしてください。

```bash
cargo install --locked cargo-about --version 0.9.2 --features cli
cargo about generate --locked --fail --output-file THIRD_PARTY_LICENSES.html about.hbs
```
