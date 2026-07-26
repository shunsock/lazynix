# システムアーキテクチャ

LazyNix は 4 つのクレートからなる Rust ワークスペースです。各クレートは単一の責務を持ち、依存関係は CLI から純粋なドメインコアへと一方向に流れます。この章では、各クレートの役割、ポートとアダプタによる接続方法、設定データの構造、そして 1 回の `lnix develop` 実行がレイヤーをどう通過するかを説明します。

## アーキテクチャ概要

ワークスペースは `crates/` 以下に構成されています:

```
crates/
  lnix/          # バイナリ: CLI エントリーポイント (clap 解析 + コンポジションルート)
  lnix-app/      # ライブラリ: ユースケース (init/develop/run/test/task/lint/update/search)
  lnix-domain/   # ライブラリ: 純粋ドメイン — 定義、サービス、ポート、値オブジェクト
  lnix-infra/    # ライブラリ: アダプタ — ファイルシステム、nix サブプロセス、nix-versions、stdout
```

設計はヘキサゴナル (「ポートとアダプタ」) レイアウトに従います。`lnix-domain` は最内層で、I/O を一切行いません。`lnix-app` は `lnix-domain` に定義されたトレイト (ポート) のみを介してユースケースをオーケストレーションします。`lnix-infra` は具象アダプタを提供し、`lnix` バイナリはすべてを配線するコンポジションルートです。

## クレート構成

### lnix

**クレート:** `lnix` (バイナリ)
**責務:** CLI 引数の解析、およびコンポジションルートとしての役割。

`lnix` は [clap](https://docs.rs/clap/) を用いて `lnix` コマンドとそのサブコマンドを定義します:

| サブコマンド | 説明 |
|------------|------|
| `init`     | 同梱テンプレートから `lazynix.yaml` と初期ファイルを生成 |
| `update`   | シェルに入らずに `flake.lock` を更新 |
| `develop`  | `flake.nix` を再生成し `nix develop` に入る |
| `run`      | 開発シェル内で単一のコマンドを実行 |
| `test`     | `lazynix.yaml` の `test` に定義されたコマンドを実行 |
| `task`     | `task` セクションに定義された名前付きタスクを実行 |
| `lint`     | チャネル由来 (stable / unstable) のパッケージを `nix eval` で検証 |
| `search`   | nix-versions を用いて利用可能なバージョンを検索 |

バイナリ自体にビジネスロジックはありません。`main.rs` は引数を解析し、`AdapterSet` (コンポジションルート) を組み立て、それらを `lnix_app::Deps` バンドルに借用させ、`lnix-app` の対応するユースケースにディスパッチします。

### lnix-app

**クレート:** `lnix-app` (ライブラリ)
**責務:** ドメインのポートに対してユースケースをオーケストレーションする。

各サブコマンドは `usecase/` 配下の `fn(&Deps, ...) -> Result<i32, ApplicationError>` 形状の関数に対応します。`Deps` は、ユースケースが触れうるすべてのポートを借用でまとめたバンドルで、`ConfigRepository`、`FlakeWriter`、`EnvFilePresenceChecker`、`ProjectScaffolder`、`NixRunner`、`NixEvaluator`、`VersionResolver`、`OutputPort` を含みます。

`flake.nix` を生成するユースケース (`develop` / `test` / `run`) は、`pipeline.rs` に定義された共通の前段を共有します:

1. `load_config` — 設定ファイル (settings) の読み込み、`lazynix.yaml` の読み込み、`validate_config` の実行 (診断は `OutputPort::warn` に流す)、参照される dotenv ファイルの存在チェック、pinned パッケージの解決とその結果の `lazynix.yaml` への書き戻し。
2. `write_flake` — `lnix_domain::render_flake` を呼び出し、結果を `./flake.nix` に書き込む。
3. `maybe_update_lock` — `--update` が指定されているときのみ `NixRunner::flake_update` を呼び出す。

エラーは鉄道パターンで合成されます。各ステップは `Result` を返し、ドメインの絞られたエラーは `#[from]` によって `ApplicationError` に持ち上げられるため、ユースケース本体は直線的なままで、`?` が失敗を `main()` まで短絡させます。

### lnix-domain

**クレート:** `lnix-domain` (ライブラリ)
**責務:** 純粋なドメイン。I/O を持たず、依存は `serde` と `thiserror` のみ。

ドメインは 4 つのサブモジュールに分かれます:

- `definition/` — 設定 AST: `DevShellDefinition`、`DevShell`、`Package`、`PackageEntry`、`PinnedPackageEntry`、`Env`、`EnvVar`、`TaskDef`、`Settings`。加えて `validate_config` は、非致命の指摘を `Diagnostic` 値として返し、致命的な違反は `ValidationError` として返します。
- `values/` — 構築時に不変条件を検証する値オブジェクト: `PackageName`、`PackageVersion`、`TaskName`、`EnvVarName`、`RegistryUrl`。下流コードで不正な値を表現不可能にすると同時に、生成される Nix 式や起動されるサブプロセスへ流れる値に対するシェルインジェクション対策も兼ねます。
- `service/` — 純粋なドメインサービス: `flake::render_flake` (`DevShellDefinition` を `flake.nix` 文字列に変換)、`lint::*` (生の `nix eval` エラーを分類して検証レポートを整形)、`task::interpolate_command` (CLI 引数をタスクテンプレートに展開)。
- `interface/` — ポート。トレイトは `interface::persistence` (`ConfigRepository`、`FlakeWriter`、`EnvFilePresenceChecker`、`ProjectScaffolder`)、`interface::gateway` (`NixRunner`、`NixEvaluator`、`VersionResolver`)、`interface::output` (`OutputPort`) に分類されます。

### lnix-infra

**クレート:** `lnix-infra` (ライブラリ)
**責務:** ドメインポートに対する具象アダプタ。

`lnix_domain::interface` で宣言されたすべてのトレイトが、ここで実装されます:

- `persistence/` — ファイルシステムアダプタ (`ConfigRepository`、`FlakeWriter`、`EnvFilePresenceChecker`、`ProjectScaffolder`)。すべてのパスは `WorkspacePaths` を起点とし、どのアダプタも暗黙にカレントディレクトリを読みません。
- `gateway/` — `nix` および `nix-versions` を呼び出すサブプロセスアダプタ。2 つの内部ヘルパー (対話コマンド用の `run_inherit` と、出力を取り込む `run_capture`) に stdio 配線とエラーマッピングを集約しています。
- `output/` — `OutputPort` を実装するターミナルシンク。

`lnix-infra` は `lnix init` で使用されるテンプレートも同梱しています。

## 依存方向と依存性逆転

依存グラフは、単一の逆転を含む直線構造です:

```
lnix  ─►  lnix-app  ─►  lnix-domain  ◄─  lnix-infra
```

- `lnix` はコンポジションルートでのみ `lnix-app` と `lnix-infra` に依存します。
- `lnix-app` は `lnix-domain` にのみ依存します。具象アダプタを名指しせず、トレイトオブジェクト (`&dyn ConfigRepository`、`&dyn NixRunner` など) に対してのみ話しかけます。
- `lnix-domain` は内部のどのクレートにも依存しません。ポートを定義する側です。
- `lnix-infra` は `lnix-domain` に依存し、そのポートを実装します。矢印が「逆向き」に描かれているのは意図的で、これがドメインを単体で試験可能に保つ **依存性逆転** です。

ポートがトレイトであるため、ユースケースのテストは `&dyn` 参照でフェイクを持たせた `Deps` を組み立てるだけで差し替えられます。本番とテストでユースケースコードは変わりません。

## データモデル

`DevShellDefinition` は `lazynix.yaml` のルートです:

```
DevShellDefinition
  └── DevShell
        ├── allowUnfree:  bool                     (デフォルト: true)
        ├── package: Package
        │     ├── stable:   Vec<PackageEntry>              # { name: PackageName }
        │     ├── unstable: Vec<PackageEntry>              # { name: PackageName }
        │     └── pinned:   Vec<PinnedPackageEntry>        # { name, version, resolvedCommit?, resolvedAttr? }
        ├── shellHook:   Vec<String>
        ├── env:         Option<Env>                       # { dotenv: Vec<String>, envvar: Vec<EnvVar> }
        ├── test:        Vec<String>
        ├── task:        Option<HashMap<TaskName, TaskDef>>
        └── shellAlias:  Vec<String>                       # エイリアス定義を読み込む対象ファイル
```

新しめのフィールドに関する補足:

- `pinned` はパッケージを厳密なバージョンに固定します。`resolvedCommit` と `resolvedAttr` は、パイプラインが `VersionResolver` 経由で初めて解決した際に埋められ、以降の実行で再解決を避けるために `lazynix.yaml` に書き戻されます。
- `shellAlias` は、シェルエイリアスの定義を開発シェルへロードする対象ファイルの一覧です。
- `env.envvar[].name` は `EnvVarName`、`task` のキーは `TaskName` の値オブジェクトで、不正な識別子は YAML パース時点で拒否されます。

中間表現は別に存在しません。`render_flake` は `DevShellDefinition` を直接走査して `flake.nix` 文字列を生成します。

## 検証ルール

`lnix_domain::validate_config` は、値オブジェクトでは表現できないフィールド間の制約を検査します。フィールド単位の不変条件 (パッケージ名の構文、バージョンの非空、タスク名・環境変数名の構文) は、serde が値オブジェクトを構築する時点で既に強制されているため、`validate_config` は残りの関係だけを検査します。

結果は 2 通りです:

- **致命的エラー** — `ValidationError::EmptyTaskCommands(name)`。タスクの `commands` が空のとき返り、パイプラインは `flake.nix` をレンダリングする前に停止します。
- **非致命の診断** — `Diagnostic::NoPackages`。`stable` / `unstable` / `pinned` がすべて空のときに返ります。`validate_config` はデータとして返し、`pipeline::load_config` が `OutputPort::warn` に転送して実行は継続します。

追加の不変条件は他の箇所で強制されます:

- `pipeline::validate_env_files` は、`env.dotenv` が参照するファイルが存在しないときに失敗します。
- 値オブジェクトのパーサー (`PackageName`、`PackageVersion`、`TaskName`、`EnvVarName`) は構文的に不正な入力をパース時に弾き、そのことが最終的に生成される Nix 式や起動されるサブプロセスへのシェルインジェクション対策も兼ねています。

## データフロー

`lnix develop` を実行したときの流れを示します:

```
ユーザーが実行: lnix develop [--update]
  │
  1. lnix (バイナリ) が clap で引数を解析。
  │
  2. lnix が AdapterSet を組み立て、lnix_app::Deps バンドルへ借用させ、
     lnix_app::develop にディスパッチ。
  │
  3. lnix_app::pipeline::load_config
       ├── ConfigRepository::read_settings         (任意の lazynix-settings.yaml)
       ├── ConfigRepository::read_config           (lazynix.yaml → DevShellDefinition)
       ├── lnix_domain::validate_config            (診断 → OutputPort::warn)
       ├── validate_env_files                      (dotenv ファイルの存在確認)
       └── resolve_pinned_packages                 (VersionResolver::resolve →
                                                    lazynix.yaml への書き戻し)
  │
  4. lnix_app::pipeline::write_flake
       └── lnix_domain::render_flake → FlakeWriter::write_flake
  │
  5. lnix_app::pipeline::maybe_update_lock          (--update 指定時のみ)
       └── NixRunner::flake_update
  │
  6. NixRunner::develop                             (`nix develop` を exec し、
                                                     現在のプロセスを置き換える)
```

各ステップは、成功して次に制御を渡すか、`Result::Err` を返して `ApplicationError` に持ち上げられ `main()` で表示されるかのいずれかです。リトライも、フォールバックも、隠れた状態もありません。

## まとめ

| クレート | 種別 | 責務 |
|---------|------|------|
| `lnix`        | バイナリ | CLI 解析とコンポジションルート |
| `lnix-app`    | ライブラリ | ユースケースとポートに対するパイプラインオーケストレーション |
| `lnix-domain` | ライブラリ | 定義、値オブジェクト、純粋サービス、ポートトレイト |
| `lnix-infra`  | ライブラリ | ファイルシステム、`nix`、`nix-versions`、stdout のアダプタ |

依存関係は `lnix` から `lnix-domain` へと一方向に流れ、`lnix-infra` はポートトレイトによる逆転した矢印を介して `lnix-domain` と接続されます。各クレートは独立して理解、テスト、変更できます。

## 補足

この概要より踏み込んだ設計文書は `document/jp/design/` 配下に置き、現状は日本語のみ提供しています。ピニング周りの詳細は例として `document/jp/design/version-pinning.md` を参照してください。
