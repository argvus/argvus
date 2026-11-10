---
title: テーマのインストール
description: pacmanを使って公式のARGVUSテーマパッケージをインストールします。
---
公式テーマは Arch Linux のパッケージとして公開されています。各テーマは `argvus-theme-<THEME_NAME>` という名前の独自のパッケージです。

## テーマの入手先

すべての公式テーマのソースは、ARGVUS プロジェクト組織にあります：

- 各テーマは `argvus-theme-<THEME_NAME>`（例：`argvus-theme-dracula`）という名前のリポジトリで、同じ名前のパッケージとして構築されます。

テーマパッケージは `argvus-appearance` に依存します。外観コンポーネントが存在するように、[インストール](/docs/getting-started/installation/) に記載されている通り、まず ARGVUS をインストールしてください。

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
| `argvus-theme-one-dark` | One Dark |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Silver Dark |
| `argvus-theme-slate-dark` | Slate Dark |
| `argvus-theme-universe` | Universe |
| `argvus-theme-gruvbox-high-dark` | Gruvbox High Dark |
| `argvus-theme-gruvbox-dark` | Gruvbox Dark |
| `argvus-theme-rose-pine` | Rosé Pine |
| `argvus-theme-tokyo-night` | Tokyo Night |
| `argvus-theme-solitude` | Solitude |
| `argvus-theme-sunset` | Sunset |
| `argvus-theme-matrix` | Matrix |
| `argvus-theme-monokai-dark` | Monokai Dark |

**ライトテーマ**

| パッケージ | テーマ |
| --- | --- |
| `argvus-theme-github-light` | GitHub ライト |
| `argvus-theme-solarized-light` | Solarized ライト |
| `argvus-theme-nord-light` | Nord ライト |
| `argvus-theme-one-light` | One ライト |
| `argvus-theme-everforest-light` | Everforest ライト |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox ライト |
| `argvus-theme-watercolor` | Watercolor |

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
