# システムアーキテクチャ

LazyNixは4つのクレートからなるRustワークスペースとして構成されています。各クレートは明確に定義された単一の責務を持ち、依存関係はCLIエントリーポイントから下位のライブラリへと一方向に流れます。この章では、各クレートの役割、それらの接続方法、コマンド実行時のデータフローを説明します。

## アーキテクチャ概要

ワークスペースは `src/` 以下に構成されています:

```
src/
  cli/              # エントリーポイントとコマンドルーティング
  flake-generator/  # YAML解析とflake.nix生成
  linter/           # nix evalによるパッケージ検証
  nix-dispatcher/   # Nixコマンドの実行
  templates/        # lnix init用のテンプレートファイル
```

依存グラフは厳密にトップダウンに流れます:

```
              cli
           /   |   \
          v    v    v
  flake-     linter   nix-
  generator             dispatcher
```

`cli` は3つのライブラリクレートすべてに依存します。ライブラリクレート同士は互いに依存しません。このフラットな依存構造により、モジュールは疎結合に保たれます。Nixコマンドの実行方法（`nix-dispatcher`）を変更しても、YAMLの解析方法（`flake-generator`）には影響せず、その逆も同様です。

## cli

**クレート:** `lnix`（バイナリ）
**責務:** CLI引数の解析、サブコマンドのオーケストレーション、他クレートの調整。

`cli` クレートはワークスペース内の唯一のバイナリです。[clap](https://docs.rs/clap/) を使って `lnix` コマンドとそのサブコマンドを定義しています:

| サブコマンド | 説明 |
|------------|------|
| `init`     | テンプレートから `lazynix.yaml` と `flake.nix` を作成 |
| `develop`  | `flake.nix` を生成しNix開発シェルに入る |
| `run`      | 開発環境内でコマンドを実行 |
| `test`     | `lazynix.yaml` で定義されたテストコマンドを実行 |
| `task`     | `lazynix.yaml` で定義された名前付きタスクを実行 |
| `update`   | `flake.lock` を更新 |
| `lint`     | `nix eval` でパッケージを検証 |

各サブコマンドは同じパターンに従います:

1. 設定を読み込む（`flake-generator` 経由）
2. 設定を検証する
3. 必要に応じて `flake.nix` を生成する（`flake-generator` 経由）
4. Nixコマンドを実行する（`nix-dispatcher` 経由）

`cli` クレートにはビジネスロジックがありません。ライブラリクレートに作業を委譲する薄い調整レイヤーです。エラーハンドリングは鉄道パターンに従います。各ステップは `Result` を返し、エラーは `main()` まで伝播し、そこで表示されて終了コードに変換されます。

### 主要モジュール

- `cli_parser.rs` --- clapのderiveマクロで `Cli` 構造体と `Commands` 列挙型を定義。
- `commands/` --- 独立したモジュールが必要なほど複雑なサブコマンドの実装（例: `lint`）。
- `env_validator.rs` --- 環境変数設定の検証（dotenvファイルの存在確認など）。
- `task_interpolator.rs` --- CLI引数をタスクコマンドテンプレートに展開。

## flake-generator

**クレート:** `lnix-flake-generator`（ライブラリ）
**責務:** `lazynix.yaml` の解析と `flake.nix` コンテンツの生成。

このクレートは、YAMLからNixへの中核的な変換を担います。3つのパブリック関数を公開しています:

```rust
// lazynix.yamlをConfig構造体に解析
let parser = LazyNixParser::new(config_dir);
let config: Config = parser.read_config()?;

// Configを検証（空のパッケージ、無効な名前などをチェック）
validate_config(&config)?;

// Configをflake.nix文字列にレンダリング
let flake_content: String = render_flake(&config, override_url);
```

### データモデル

`Config` 構造体は `lazynix.yaml` の構造を反映しています:

```
Config
  └── DevShell
        ├── allow_unfree: bool
        ├── Package { stable, unstable }
        ├── shell_hook: Vec<String>
        ├── env: Env { dotenv, envvar }
        ├── test: Vec<String>
        └── task: HashMap<String, TaskDef>
```

パーサーは [serde](https://serde.rs/) を使ってYAMLをこの構造体にデシリアライズします。ジェネレーターはこの構造体を走査して `flake.nix` 文字列を生成します。中間表現はありません。データモデルから出力文字列への変換は直接的です。

### 検証

`validate_config` は、YAMLの構文だけでは強制できない制約をチェックします:

- 少なくともひとつのパッケージが宣言されていること
- パッケージ名が空文字列でないこと
- stableとunstableにまたがる重複パッケージ名の検出

検証は生成の前に実行されるため、無効な設定が `flake.nix` を生成することはありません。

## linter

**クレート:** `lnix-linter`（ライブラリ）
**責務:** ユーザーがシェルに入る前に、宣言されたパッケージがnixpkgsに存在することを検証する。

リンターは `nix eval` を使って、`lazynix.yaml` の各パッケージが解決可能かどうかをチェックします。タイプミスやプラットフォーム非互換のパッケージを早期にキャッチし、遅い `nix develop` の実行が失敗するのを待つ必要をなくします。

```
入力: パッケージ名 + ターゲットアーキテクチャ
  │
  ├── nix_eval::eval_package()     # 各パッケージに対してnix evalを実行
  ├── error_classifier::classify() # nix evalの失敗を分類
  ├── validator::validate()        # 結果を集約
  └── reporter::format()           # 人間が読める出力をフォーマット
```

### 主要な設計判断

- **並列評価。** リンターは [rayon](https://docs.rs/rayon/) を使って複数パッケージを並行に評価します。`lazynix.yaml` が多くのパッケージを宣言している場合に重要です。
- **エラー分類。** 生の `nix eval` エラーをカテゴリ（パッケージが見つからない、属性パスエラー、アーキテクチャ非互換）に解析し、Nix評価出力の壁ではなくアクション可能なメッセージをユーザーに提供します。
- **アーキテクチャ対応。** デフォルトではリンターは現在のシステムアーキテクチャでパッケージをチェックします。`--arch` フラグにより、異なるターゲットでのチェックが可能です（例: `aarch64-darwin` マシンから `x86_64-linux` 向けの設定が動作するか検証）。

## nix-dispatcher

**クレート:** `lnix-nix-dispatcher`（ライブラリ）
**責務:** Nixコマンドをサブプロセスとして実行する。

このクレートは、LazyNixとNix CLIの間にクリーンなインターフェースを提供します。Nixプロセスの起動、終了コードの処理、エラーの報告に関する詳細を抽象化します。

パブリックAPIは関数のセットです:

| 関数 | 動作 |
|------|------|
| `run_nix_develop()` | 対話的な `nix develop` シェルに入る |
| `run_nix_develop_command(args)` | `nix develop` 内で単一コマンドを実行 |
| `run_flake_update()` | `nix flake update` を実行 |
| `run_nix_test()` | `LAZYNIX_TEST_MODE=1` でテストコマンドを実行 |
| `run_task_in_nix_env(commands)` | 複数コマンドを順次実行 |

各関数は適切な `nix` コマンドを構築し、サブプロセスとして起動し、終了コードを返します。このクレートはNixコマンドの出力を解釈しません。成功か失敗かだけを扱います。

### エラーハンドリング

すべての関数は `Result<T, NixDispatcherError>` を返します。エラー型は2つのケースをカバーします:

- **コマンドが見つからない。** `nix` バイナリが `PATH` 上にない。
- **実行失敗。** サブプロセスを起動できなかった（権限拒否など）。

`nix` からのゼロ以外の終了コードは、Rustの意味でのエラーとして扱われないことに注意してください。値（`i32`）として返されるため、呼び出し側がその処理を決定できます。典型的には `lnix` プロセスの終了コードとしてそのまま転送されます。

## データフロー

全体を結びつけるために、`lnix develop` を実行したときに何が起こるかを示します:

```
ユーザーが実行: lnix develop
  │
  1. cli が引数を解析（clap）
  │
  2. cli が lazynix-settings.yaml を読み込み（任意）
  │   └── stableパッケージに使用するnixpkgsバージョンを
  │       上書きするためのオプションファイル（例: カスタムフォークの指定）。
  │       ほとんどのユーザーはこのファイルを必要としない。
  │
  3. flake-generator が lazynix.yaml を読み込み
  │   └── Config構造体にデシリアライズ
  │
  4. flake-generator が Config を検証
  │   └── パッケージが空または無効な場合はエラーを返す
  │
  5. cli が env設定を検証
  │   └── 参照された .env ファイルの存在を確認
  │
  6. flake-generator が flake.nix をレンダリング
  │   └── 生成した文字列を ./flake.nix に書き込み
  │
  7. nix-dispatcher が nix flake update を実行（--update 指定時）
  │
  8. nix-dispatcher が nix develop を実行
  │   └── 現在のプロセスを対話的シェルに置き換え
  │
  ユーザーは開発環境の中にいる。
```

各ステップは成功して次に制御を渡すか、エラーを返して `main()` まで伝播させるかのどちらかです。リトライも、フォールバックも、隠れた状態もありません。フローは線形で予測可能です。

## まとめ

| クレート | 種別 | 責務 |
|---------|------|------|
| `cli` | バイナリ | 引数解析、コマンドオーケストレーション |
| `flake-generator` | ライブラリ | YAML解析、検証、Nixコード生成 |
| `linter` | ライブラリ | `nix eval` によるパッケージ存在検証 |
| `nix-dispatcher` | ライブラリ | サブプロセスとしてのNixコマンド実行 |

依存関係は一方向に流れます。ライブラリクレート同士は互いに依存しません。各クレートは独立して理解、テスト、変更できます。
