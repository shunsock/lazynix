# はじめよう

LazyNixへようこそ! このガイドでは、再現可能な開発環境を最初からセットアップする手順を説明します。ガイドを読み終える頃には、チームの誰もがコマンドひとつでまったく同じツールを、まったく同じバージョンで使えるプロジェクトが手に入ります。

Nixの事前知識は不要です。LazyNixがNixの複雑さを裏側で処理するので、あなたはコードを書くことに集中できます。

## Nixのインストール

LazyNixは[Nix](https://nixos.org/)というパッケージマネージャの上に構築されています。Nixは再現可能なビルドを保証するツールです。LazyNixを使う前に、Nixをインストールしてください。

[nixos.org/download](https://nixos.org/download/) にアクセスし、お使いのOSの手順に従ってください。インストール後、動作を確認します:

```bash
nix --version
```

`nix (Nix) 2.x.x` のような出力が表示されれば成功です。

LazyNixはNix Flakesを使用しており、`flakes` と `nix-command` の実験的機能を有効にする必要があります。Nixの設定ファイル（`~/.config/nix/nix.conf`）に以下を追加してください:

```
experimental-features = nix-command flakes
```

ファイルが存在しない場合は新規作成してください。保存後、`nix --version` を再度実行してNixが正常に動作することを確認します。Nixのセットアップはこれだけです。残りはLazyNixが処理します。

## LazyNixのインストール

LazyNixを試す最も簡単な方法は、インストールせずにGitHubから直接実行することです:

```bash
nix run github:shunsock/lazynix -- --help
```

日常的に使う場合は、Nixプロファイルにインストールして `lnix` コマンドを常に利用できるようにします:

```bash
nix profile install github:shunsock/lazynix
```

インストールを確認します:

```bash
lnix --help
```

> **補足:** インストールしたくない場合は、このガイドのすべてのコマンドで `lnix` を `nix run github:shunsock/lazynix --` に置き換えられます。

## 最初のプロジェクトを作る

プロジェクトディレクトリに移動し（または新しく作成し）、以下を実行します:

```bash
lnix init
```

このコマンドは2つのファイルを生成します:

- `lazynix.yaml` --- 環境設定ファイル。編集するのはこのファイルだけです。
- `flake.nix` --- 生成されたNix flake。LazyNixが自動管理します。

生成された `lazynix.yaml` を見てみましょう:

```yaml
devShell:
  allowUnfree: false
  package:
    stable:
      - hello
    unstable: []
  shellHook:
    - "echo Welcome to LazyNix DevShell!"
```

これが設定のすべてです。各フィールドの意味を確認しましょう:

- **`allowUnfree`** --- Nixはオープンソースとプロプライエタリ（unfree）パッケージを区別しています。VS CodeやCUDAツールキットなどのプロプライエタリソフトウェアが必要な場合は `true` に設定します。`false` の場合、オープンソースパッケージのみ許可されます。
- **`package.stable`** --- [nixpkgs](https://github.com/NixOS/nixpkgs)（Nixのパッケージリポジトリ）の安定版スナップショットから取得されるパッケージです。ほとんどのツールにはこちらを使います。
- **`package.unstable`** --- nixpkgsの最新版から取得されるパッケージです。安定チャネルにまだ到達していない最新バージョンが必要な場合に使います。
- **`shellHook`** --- 開発環境に入るたびに自動実行されるシェルコマンドです。バージョン情報の表示、エイリアスの設定、初期化スクリプトの実行などに便利です。

Nixの構文を覚える必要はありません。YAMLだけです。

## 環境をカスタマイズする

Pythonプロジェクトを始めるとしましょう。パッケージマネージャには [uv](https://docs.astral.sh/uv/) を使います。`lazynix.yaml` を開いて、内容を次のように書き換えます:

```yaml
devShell:
  allowUnfree: false
  package:
    stable:
      - python312
      - uv
    unstable: []
  shellHook:
    - "echo Python $(python --version) ready!"
    - "echo uv $(uv --version) ready!"
```

`python312` と `uv` を `package.stable` に配置しました。安定チャネルで十分な、実績のあるツールだからです。シェルフックは、環境に入るたびにツールが利用可能であることを確認します。

パッケージ名の調べ方は簡単です。[search.nixos.org](https://search.nixos.org/packages) にアクセスし、必要なツールを検索してください。表示されるパッケージ名が、そのまま `lazynix.yaml` に書く名前です。

## 開発環境に入る

設定した環境に入りましょう:

```bash
lnix develop
```

LazyNixは `lazynix.yaml` を読み込み、`flake.nix` を生成（既存のものは上書き）し、Python 3.12とuvが使えるシェルに入ります。以下のような出力が表示されます:

```
Reading configuration from .
Validating configuration...
Generating flake.nix...
✓ flake.nix generated successfully
Skipping flake.lock update (use --update to update)

Python Python 3.12.x ready!
uv uv 0.x.x ready!
```

チームの誰が `lnix develop` を実行しても、同じPythonバージョン、同じuvバージョンが手に入ります。「自分のマシンでは動く」問題はもう起きません。

固定されたパッケージバージョンを更新するには、`--update` フラグを付けます:

```bash
lnix develop --update
```

これにより `flake.lock` が更新されます。`flake.lock` は、すべてのパッケージの正確なバージョンを固定するロックファイルです。`--update` なしでは、LazyNixは `flake.lock` に記録されたバージョンを使用し、すべての開発者が同一のツールを手に入れることを保証します。`--update` を付けると、nixpkgsから最新バージョンを取得します。

> **重要:** `lnix develop` は実行のたびに `lazynix.yaml` から `flake.nix` を再生成します。`flake.nix` を手動で編集していた場合、その変更は上書きされます。この設計判断の詳細と、純粋なNixへの移行方法については [設計思想](./philosophy.md) を参照してください。

## コマンドを実行する

対話的なシェルに入る必要がない場合もあります。環境内でコマンドをひとつだけ実行したいときは `lnix run` を使います:

```bash
lnix run -- python -c "print('Hello from LazyNix!')"
```

`--` はLazyNixのフラグと実行したいコマンドを区切ります。`--` 以降のすべてがNix開発環境の中で実行されます。

## タスクを定義する

繰り返し実行するコマンドは、`lazynix.yaml` で名前付きタスクとして定義できます:

```yaml
devShell:
  allowUnfree: false
  package:
    stable:
      - python312
      - uv
    unstable: []
  shellHook:
    - "echo Python $(python --version) ready!"

  task:
    test:
      description: "テストスイートを実行する"
      commands:
        - "python -m pytest"
    fmt:
      description: "コードをフォーマットする"
      commands:
        - "uv run ruff format ."
```

タスクは名前で実行します:

```bash
lnix task test
lnix task fmt
```

タスクは開発環境内で順番に実行されるため、宣言したすべてのパッケージが利用可能です。

## テストを実行する

プロジェクトにテストコマンドがある場合、`lazynix.yaml` の `test` フィールドに直接定義できます:

```yaml
devShell:
  package:
    stable:
      - python312
      - uv
    unstable: []

  test:
    - "python -m pytest"
    - "python -m mypy src/"
```

すべてのテストを実行するには:

```bash
lnix test
```

開発環境に入り、各テストコマンドを順番に実行します。いずれかのコマンドが失敗すると、`lnix test` はゼロ以外のステータスコードで終了します。

`test` と `task` の違いは意図の違いです。`test` はCIパイプラインやpre-commitチェック向けに設計されたコマンドのフラットなリストで、`task` は説明文や引数の展開を持つ、名前付きの再利用可能なワークフローを定義します。

## 設定を検証する

設定をコミットする前に、宣言したすべてのパッケージがnixpkgsに実際に存在するか検証できます:

```bash
lnix lint
```

`nix eval` を使って各パッケージをチェックし、見つからないパッケージを報告します。パッケージ名のタイプミスや存在しないパッケージを、ビルド時に分かりにくいエラーとなる前にキャッチします。

## ここまでに学んだこと

このガイドでは、以下を行いました:

- 唯一の前提条件であるNixをインストールした
- `lnix init` でLazyNixプロジェクトを作成した
- `lazynix.yaml` でPython開発環境を設定した
- `lnix develop` で環境に入った
- `lnix run` でコマンドを実行し、再利用可能なタスクを定義した
- `lnix lint` で設定を検証した

## 次のステップ

- [設計思想](./philosophy.md) を読んで、LazyNixの設計判断の背景を理解する
- [システムアーキテクチャ](./system_architecture.md) を読んで、LazyNixの内部構造を学ぶ
- [examples](../../examples/) ディレクトリで、さらなる設定パターンを探索する
