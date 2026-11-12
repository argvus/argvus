---
title: ARGVUSをインストールする
description: ARGVUSをインストール、更新、削除、そして設定します。
---
ARGVUSは、公式のARGVUSパッケージリポジトリを通じて、Arch Linuxのパッケージとして配布されています。

## インストール前に

ARGVUSをインストールする前に、公式のArch Linux `multilib`リポジトリを有効にしてください。`/etc/pacman.conf`を編集し、`[multilib]`セクションとその`Include`行の両方のコメントを外します:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
次に、パッケージデータベースを同期させ、システム全体のアップグレードを実行します:

```sh
sudo pacman -Syu
```
クイックインストーラーはARGVUSリポジトリを設定しますが、`multilib`を有効化することはしません。

## クイックインストール

新しいArch Linuxのインストールでは、最初に他のデスクトップ環境をインストールせずに、TTYから直接ARGVUSをインストールできます。

実行:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
インストーラー:

1. ARGVUSリポジトリの署名キーをインポートしてローカルで署名します。
2. ARGVUSパッケージリポジトリを設定します。
3. パッケージデータベースを更新します。
4. `argvus`パッケージとその依存関係をインストールします。

インストール後、ログアウトしてディスプレイマネージャーから**ARGVUS**を選択してください。

> **注意:** クイックインストーラーには`curl`、`sudo`、そして標準のArch Linuxパッケージ管理ツールが必要です。

## リポジトリのみを設定する

デスクトップをインストールせずにARGVUSパッケージリポジトリを設定するには、次のコマンドを実行してください:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
その後、ARGVUSを手動でインストールします:

```sh
sudo pacman -Syu argvus
```
## 手動インストール

リポジトリをステップごとに設定したい場合は、次のコマンドを使用してください。

### 1. リポジトリ署名キーのダウンロード

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. 署名キーをインポートする

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. ローカルで鍵に署名する

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. ARGVUSリポジトリを設定する

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
リポジトリの設定を `/etc/pacman.conf` に追加してください：

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
リポジトリの設定により、SigLevel = Required TrustedOnlyと共に`argvus`および`argvus-extras`を含むARGVUSリポジトリが有効になります。Pacmanは、署名が有効であり、ローカルの鍵リングにより信頼されている場合にのみパッケージを受け入れます。

### 5. ARGVUSのインストール

```sh
sudo pacman -Syu argvus
```
インストール後、ログアウトしてディスプレイマネージャーから **ARGVUS** を選択してください。

## 更新

他の Arch Linux と一緒に ARGVUS を更新します:

```sh
sudo pacman -Syu
```
ARGVUS パッケージのみを更新するには:

```sh
sudo pacman -S argvus
```

> **推奨:** Arch Linuxでは、個別のパッケージを更新する代わりに、 `SigLevel = Required TrustedOnly` `sudo pacman -Syu`でシステム全体のアップグレードを行うことが一般的に推奨されています。

## ARGVUS を削除

他のインストールされているパッケージでまだ必要とされている依存関係を保持しながら、ARGVUS を削除します:

```sh
sudo pacman -R argvus
```
インストールされているどのパッケージからも不要になった ARGVUS とその依存関係を削除します:

```sh
sudo pacman -Rns argvus
```
### ARGVUSリポジトリを削除する

パッケージリポジトリも削除したい場合は、その設定を削除してください:

```sh
sudo rm /etc/pacman.d/argvus.conf
```
次に、`/etc/pacman.conf` から対応する行を削除してください:

```text
Include = /etc/pacman.d/argvus.conf
```
次の方法でファイルを編集できます:

```sh
sudo nano /etc/pacman.conf
```
### 署名キーを削除する

pacman キーリングから ARGVUS リポジトリの署名キーを削除するには、まずキーを一覧表示します:

```sh
sudo pacman-key --list
```
次に、ARGVUSキーとそのキーIDを削除します:

```sh
sudo pacman-key --delete KEY_ID
```
`KEY_ID` を ARGVUS の署名キー ID に置き換えてください。

> **警告:** このキーは、ARGVUS リポジトリを削除する場合にのみ削除してください。他のリポジトリでまだ必要なキーを削除しないでください。

## ユーザー設定

パッケージのインストールは `$HOME` に設定を書き込みません。ARGVUS は `/usr/share/argvus` からパッケージ化されたデフォルトを読み取り、それらをホームディレクトリにコピーせずに動作します。

ランタイム設定は、一般的に以下の優先順位に従います:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config` は `~/.config/argvus/data/generated/` を書き込む唯一のコンポーネントであり、そのプロジェクションはアクティブレイヤーです：管理されたコピーまたはネイティブコピーは、対応する生成ファイルが存在しない場合にのみ有効になります。

ARGVUS 管理の設定は `~/.config/argvus/` の下に存在します。オプションの `$XDG_CONFIG_HOME/<app>` オーバーライドは、ネイティブユーザー設定をサポートするアプリケーション向けに引き続き利用可能です。

`argvus --setup` ディスパッチャーはオプションであり、明示的な手動オーバーライド用であり、リカバリー用ではありません。`~/.config/argvus` が削除されると、`SUPER + SHIFT + R` は通常のリロードパイプラインを通じて、標準プロファイルと生成されたランタイム状態を再作成します。

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all` はサポートされている TUI プロフィールファイルの明示的な許可リストです。`/usr/share/argvus` をクローンしたり、スクリプト、サービス、アセット、生成ファイル、または `config.json` をコピーしたりすることはありません。現在のオプションを確認するには `argvus --setup --help` を実行してください。一度コピーされたファイルはユーザー所有のオーバーライドとなり、パッケージのアップグレードでは置き換えられません。

既存のコピー済み設定を置き換えるには、`--force` を使用します。ARGVUS は置き換える前にタイムスタンプ付きのバックアップを作成します:

```sh
argvus --setup --copy <app> --force
```
## Hyprland ユーザーオーバーライド

完全な Hyprland オーバーライドガイドについては、フルネイティブ構成と増分 ARGVUS フラグメントの違いを含めて、[Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/) を参照してください。

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### 仮想マシンのフォールバック

一部の仮想マシンでは、Hyprland、Kitty、または Quickshell でレンダリングの問題が発生することがあります。ARGVUS は `vmwgfx` グラフィックスドライバーを使用して仮想マシンを検出し、必要に応じてソフトウェアレンダリングを有効にします。

仮想マシンで検出が機能しない場合は、`user.lua` でフォールバックを手動で有効にしてください:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
`user.lua` を作成または変更した後、Hyprland からログアウトし、再度ログインして、セッションで起動されるアプリケーションに変数が適用されるようにしてください。

次の方法で確認します:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
期待される出力は次のとおりです：

```text
1
```
> **注意:** `LIBGL_ALWAYS_SOFTWARE=1` は Mesa/OpenGL アプリケーションにソフトウェアレンダリングを強制的に使用させます。通常、`llvmpipe` を通じて行われます。これはグラフィック性能を低下させる可能性があるため、影響を受ける仮想マシンのフォールバックとして使用するべきです。
