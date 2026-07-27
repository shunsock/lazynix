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
  allowUnfree: true
  package:
    stable:
      - name: hello
    unstable: []
    pinned: []
  shellHook:
    - "echo Welcome to LazyNix DevShell!"
```

これが設定のすべてです。各フィールドの意味を確認しましょう:

- **`allowUnfree`** --- Nixはオープンソースとプロプライエタリ（unfree）パッケージを区別しています。デフォルトは `true` で、VS CodeやCUDAツールキットなどのプロプライエタリソフトウェアを利用できます。`false` に設定すると、オープンソースパッケージのみに制限されます。
- **`package.stable`** --- [nixpkgs](https://github.com/NixOS/nixpkgs)（Nixのパッケージリポジトリ）の安定版スナップショットから取得されるパッケージです。ほとんどのツールにはこちらを使います。
- **`package.unstable`** --- nixpkgsの最新版から取得されるパッケージです。安定チャネルにまだ到達していない最新バージョンが必要な場合に使います。
- **`shellHook`** --- 開発環境に入るたびに自動実行されるシェルコマンドです。バージョン情報の表示、エイリアスの設定、初期化スクリプトの実行などに便利です。

Nixの構文を覚える必要はありません。YAMLだけです。

## 環境をカスタマイズする

Pythonプロジェクトを始めるとしましょう。パッケージマネージャには [uv](https://docs.astral.sh/uv/) を使います。`lazynix.yaml` を開いて、内容を次のように書き換えます:

```yaml
devShell:
  allowUnfree: true
  package:
    stable:
      - name: python312
      - name: uv
    unstable: []
    pinned: []
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
  allowUnfree: true
  package:
    stable:
      - name: python312
      - name: uv
    unstable: []
    pinned: []
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
      - name: python312
      - name: uv
    unstable: []
    pinned: []

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

## パッケージバージョンを固定する

`stable` や `unstable` チャネルでは、必要な正確なバージョンが手に入らないことがあります。たとえば本番CIイメージに合わせて Go 1.21.13 でチーム全員をそろえたいのに、nixpkgs stable はすでに次のリリースへ進んでしまった、という状況です。`pinned` フィールドはこの問題を解決し、チャネルとは独立にパッケージを特定バージョンへ固定します。

Go 1.21.13 を固定する手順:

1. 利用可能なバージョンを検索します（詳しくは次の節を参照）:

```bash
lnix search go -v '>=1.21,<1.22'
```

2. 使いたいバージョンを `devShell.package.pinned` に追加します:

```yaml
devShell:
  package:
    stable:
      - name: python312
    pinned:
      - name: go
        version: "1.21.13"
```

3. `lnix develop` を実行します。LazyNix は `nix-versions` を呼び出して、そのバージョンに対応する nixpkgs のコミットハッシュと Nix の attribute path を解決し、解決したコミットを生成される `flake.nix` の入力 URL に埋め込みます。以降の実行はその `flake.nix` を直接読むため、`nix-versions` の呼び出しは (name, version) の組ごとに最大 1 回で済みます。`lazynix.yaml` 自体には手を加えません — `flake.nix` **こそがキャッシュ**です。

チームの誰もが正確に Go 1.21.13 を手に入れます。ネットワーク参照の繰り返しもドリフトも起こりません。

> **補足:** `lnix lint` は `stable`・`unstable`・`pinned` のすべてのパッケージを検証します。`pinned` エントリごとに `lint` は `nix-versions` へ問い合わせて指定されたバージョンが現在も解決可能かを確認します。バージョン指定のタイプミスは次の `lnix develop` を待たずにここで検出されます。

## 利用可能なバージョンを検索する

パッケージを固定する前に、nixpkgs にどのバージョンが存在するかを知る必要があります。それが `lnix search` の役割です。

パッケージの既知のバージョンをすべて表形式で表示します:

```bash
lnix search go
```

semver 制約でフィルタします:

```bash
lnix search go -v '>=1.21,<1.22'
```

機械可読な出力を得ます。スクリプトやCIに便利です。`--one` は最新の1件だけを返します:

```bash
lnix search go --json --one
```

典型的なワークフローは、`lnix search` で正確なバージョンを見つけ、それを前節で説明した `devShell.package.pinned` に貼り付けることです。

## flake.lock を更新する

`lnix develop --update` は `flake.lock` を更新してから開発シェルに入ります。ロックファイルの更新だけを行いたい場合 — たとえば CI で lock の変更を commit するだけで開発シェルに入らないとき — は `lnix update` を使います:

```bash
lnix update
```

このコマンドは `nix flake update` を実行してすぐ終了します。対話シェルを起動せず、長時間動くプロセスも作らない軽量なコマンドなので、自動化パイプラインや pre-commit フックに自然に組み込めます。

lock を更新してすぐに作業を始めたいときは `lnix develop --update` を、lock 更新自体が目的のときは `lnix update` を使ってください。

## シェルエイリアスの読み込み

`alias ll='ls -la'` のようなシェルエイリアスは便利ですが、`shellHook` の中に直接書くと役割が入り混じります。`devShell.shellAlias` を使うと、外部のエイリアスファイルを指定でき、LazyNix はその中の `alias …` 行を抽出して開発シェル起動時に評価します。

```yaml
devShell:
  package:
    stable:
      - name: bash
  shellAlias:
    - ./aliases.sh
    - ~/.bash_aliases
```

パスの解決は flake 生成時ではなく、シェル起動時に行われます:

- 相対パス（例: `./aliases.sh`）は `$PWD` を基準に解決されます。
- `~/` は `$HOME` に展開されます。
- 絶対パスはそのまま使われます。

指定したファイルが起動時に存在しない場合、LazyNix は黙って無視します。個人用のオプションのエイリアスファイル（`~/.bash_aliases` など）を気軽に列挙できるので、そのファイルを持たないチームメイトの環境を壊しません。

## ここまでに学んだこと

このガイドでは、以下を行いました:

- 唯一の前提条件であるNixをインストールした
- `lnix init` でLazyNixプロジェクトを作成した
- `lazynix.yaml` でPython開発環境を設定した
- `lnix develop` で環境に入った
- `lnix run` でコマンドを実行し、再利用可能なタスクを定義した
- `lnix lint` で設定を検証した
- `devShell.package.pinned` で個別のパッケージを特定バージョンへ固定し、解決された nixpkgs コミットを LazyNix に `flake.nix` へ埋め込ませた
- `lnix search` で利用可能なバージョンを検索した
- `lnix update` で `flake.lock` の更新だけを行った
- `devShell.shellAlias` で外部ファイルからシェルエイリアスを読み込んだ

## 次のステップ

- [設計思想](./philosophy.md) を読んで、LazyNixの設計判断の背景を理解する
- [システムアーキテクチャ](./system_architecture.md) を読んで、LazyNixの内部構造を学ぶ
- [examples](../../examples/) ディレクトリで、さらなる設定パターンを探索する
