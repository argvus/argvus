---
title: テーマのインストール
description: pacmanを使って公式のARGVUSテーマパッケージをインストールします。
---
公式テーマは Arch Linux のパッケージとして公開されています。各テーマは `argvus-theme-<THEME_NAME>` という名前の独自のパッケージです。

## テーマの入手先

すべての公式テーマのソースは、ARGVUS プロジェクト組織にあります：

- 各テーマは `argvus-theme-<THEME_NAME>`（例：`argvus-theme-dracula`）という名前のリポジトリで、同じ名前のパッケージとして構築されます。

テーマパッケージは `argvus-appearance` に依存します。外観コンポーネントが存在するように、[インストール](/docs/getting-started/installation/) に記載されている通り、まず ARGVUS をインストールしてください。


Git インストールがテーマファイルを読み込む場所は `src/` のみです。インストールに `packaging/` は不要で、Arch Linux パッケージの作成に使います。

## 単一のテーマをインストールする

すべての公式テーマを一度にインストールするには、`argvus-themes` メタパッケージをインストールしてください：

```bash
sudo pacman -S argvus-themes
```
すべての公開されている`argvus-theme-*`パッケージに依存します。ARGVUS DarkとARGVUS Lightはすでに`argvus-appearance`によって出荷されています。
パッケージ名でテーマを1つインストールしてください:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
例えば：

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
このパッケージはテーママニフェスト `/usr/share/argvus/themes/<theme-id>/theme.toml`（名前、カテゴリ、壁紙、パレット、およびオプションの `overrides/`）と、テーマの壁紙を `/usr/share/backgrounds/argvus/` にインストールします。テーマが適用されると、各コンポーネントのファイルはパレットからレンダリングされます。ホームディレクトリには書き込まれず、現在のテーマも変更されません。その後、コントロールセンターから選択してください。[公式テーマ](/docs/components/argvus-themes/official-themes/) を参照します。

## 複数のテーマをインストールする

すべてのテーマをインストールするメタパッケージはありません。インストールしたいテーマを一度のコマンドでインストールしてください:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## 利用可能なパッケージ

以下のパッケージは、本日公開された公式テーマパッケージです。

**ダークテーマ**

| パッケージ | テーマ |
| --- | --- |
| `argvus-theme-one-dark` |ワンダーク |
| `argvus-theme-dracula` |ドラキュラ |
| `argvus-theme-silver-dark` |シルバーダーク |
| `argvus-theme-slate-dark` |スレートダーク |
| `argvus-theme-universe` |宇宙 |
| `argvus-theme-gruvbox-high-dark` | Gruvbox ハイダーク |
| `argvus-theme-gruvbox-dark` | Gruvbox ダーク |
| `argvus-theme-rose-pine` |ローズパイン |
| `argvus-theme-tokyo-night` |東京の夜 |
| `argvus-theme-solitude` |孤独 |
| `argvus-theme-sunset` |日没 |
| `argvus-theme-matrix` |マトリックス |
| `argvus-theme-monokai-dark` |モノカイダーク |

**ライトテーマ**

| パッケージ | テーマ |
| --- | --- |
| `argvus-theme-github-light` | GitHub ライト |
| `argvus-theme-solarized-light` | Solarized ライト |
| `argvus-theme-nord-light` | Nord ライト |
| `argvus-theme-one-light` | One ライト |
| `argvus-theme-everforest-light` | Everforest ライト |
| `argvus-theme-catppuccin-latte` |キャットプッチンラテ |
| `argvus-theme-gruvbox-light` | Gruvbox ライト |
| `argvus-theme-watercolor` |水彩画 |

内蔵の **ARGVUS Dark** と **ARGVUS Light** テーマは別のパッケージを必要とせず、`argvus-appearance` に含まれています。

## インストールされているものを確認

次のコマンドでインストールされているテーマパッケージを一覧表示します:

```bash
pacman -Qs argvus-theme
```
コントロールセンターを開き、**表示 → テーマ → 公式テーマ** に移動して、セレクターで現在利用可能なテーマを確認します。そのリストに欠けているがインストールされているテーマは、パッケージの問題です。[テーマガイドのトラブルシューティングセクション](/docs/components/argvus-themes/) を参照してください。

## 自分のテーマを追加する

テーマはマニフェストであるため、自分のテーマにはパッケージは不要です。
作成してください `~/.local/share/argvus/themes/<id>/theme.toml`、ここでディレクトリ名がテーマID（小文字のアルファベット、数字、単一のハイフン）になります。

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
`bg`、`fg`、`accent`のみが必要です：他のすべての色はそれらから派生したデフォルトがあり、マニフェストでは変更するためだけに1つを定義します。これを確認し、他のテーマと同じように適用してください：

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` は、あなたのテーマを公式のテーマの横に表示します（同じ ID の公式テーマが優先されます）。あなたのテーマはテーマメニュー (`SUPER + Shift + T`) に表示されます。ログイン前のグリーターはあなたのホームディレクトリを読むことができないため、あなたのテーマが有効な間は、最後に適用した公式テーマを保持します。

古い ARGVUS バージョンが `~/.config/argvus/data/<app>/themes/` に書き込んだテーマごとのコピーは、もはや読み込まれません。アップデート後の最初のテーマ切り替えで、それらは `~/.config/argvus/data/backups/theme-copies-<time>/` に移動されます。編集したファイルを保持したい場合は、`overrides/` の自分のテーマに入れ、マニフェストの `[overrides]` テーブルで名前を付けてください。

## テーマを削除する

単一のテーマパッケージを削除するには:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
テーマパッケージを削除しても、インポートしたプロファイルは削除されません。削除するテーマが現在適用されている場合は、削除する前にコントロールセンターから別のテーマを選択してください。

## skeleton でテーマを作成

互換性のあるテーマを作るには `argvus-skeleton-theme` リポジトリから始めます。`make new ID=ocean-night NAME="Ocean Night" CATEGORY=dark` を実行し、配色と壁紙を編集します。`make check` で検証し、`make try` で試します。リポジトリ公開後は `argvus theme install URL` でインストールできます。テーマファイルは `src/` にあり、`packaging/` は Arch パッケージ作成用です。

```sh
git clone https://github.com/argvus/argvus-skeleton-theme.git argvus-theme-ocean-night
cd argvus-theme-ocean-night
make new ID=ocean-night NAME="Ocean Night" CATEGORY=dark
make check
make try
argvus theme apply ocean-night
```

## Git からテーマをインストール

既存の Git 認証情報で HTTPS または SSH 経由の skeleton リポジトリをインストールします。ユーザー単位でインストールされ、自動適用されません。`--ref` はブランチやタグ、`--replace` は既存のカスタムテーマの置換を指定します。インストール済みの公式 ID は保護されます。

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

テーマは **Custom themes** に表示されます。`r` で更新します。CLI や Control Center で使用中のテーマを削除する前に、別のテーマを適用してください。置換後は再適用が必要です。

`src/usr/share/argvus/themes/<id>/` に ID が一致する schema 2 テーマが一つ必要です。画像はテーマ内か `src/usr/share/` にコミットしてください。スクリプトは実行されません。サブモジュールと Git LFS は非対応です。`git` と SSH 用の `openssh` が必要です。保存先は `$XDG_DATA_HOME/argvus/themes/<id>`。公式パッケージは pacman で管理します。
