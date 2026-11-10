---
title: サービス
description: ARGVUS の systemd およびユーザーサービス。
---
主なユーザー対象は`argvus-session.target`です。一般的なユーザーサービスには`argvus-taskbar.service`、`argvus-control-panel.service`、`argvus-widget-telemetry.service`、`argvus-hyprland.service`、`argvus-dunst.service`、`argvus-wallpaper.service`、`argvus-hypridle.service`、クリップボードサービス、および`argvus-session-loading.service`が含まれます。

ファイアウォールは`argvus-firewall.service`という名前のシステムサービスです。カレンダーパッケージは`argvus-taskbar-calendar.service`をユーザー単位としてインストールし、`argvus-clock.service`（デフォルトで無効）は「セッションと共に開始」がオンのとき、ログイン後にARGVUS Clockを非表示で起動します。

アクティブなユニットを確認するには:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## セッション所有ユニット

セッションパッケージは、これらのユーザーユニットを `/usr/lib/systemd/user` の下にインストールします（ソースペイロードはセッションパッケージの共有構成ツリーの下にあります）：

| ユニット | 役割 |
| --- | --- |
| `argvus-session.target` | ARGVUS グラフィカルセッションのライフタイム境界。 |
| `argvus-session-prepare.service` | ワンショットの起動準備および生成された構成。 |
| `argvus-control-panel.service` | クイックシェルコントロールパネル。 |
| `argvus-taskbar.service` | メイン ARGVUS タスクバー。 |
| `argvus-widget-telemetry.service` | オプションのテレメトリ/シス情報画面。 |
| `argvus-hyprland.service` | `hyprctl reload` を実行するオンデマンドのワンショットユニット。 |
| `argvus-config.service` | 標準構成（`argvus config project`）を投影；リロード時に `argvus sessionctl reload-config-transaction` を実行。 |
| `argvus-session-reload.service` | ログインリロード (`argvus sessionctl reload-desktop --login`)：プロジェクションに影響があった場合にのみ Hyprland をリロードします。 |
| `argvus-ssh-agent.service` | セッション用の SSH エージェント。 |
| `argvus-secret-service.service` | `argvus sessionctl start-secret-service` を通じてシークレットサービスを起動します。 |
| `argvus-wallpaper.service` | アクティブな壁紙バックエンド。 |
| `argvus-dunst.service` | Dunst 通知デーモン。 |
| `argvus-hypridle.service` | アイドルおよびロックポリシー。 |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | クリップボード履歴ウォッチャー。 |
| `argvus-keyboard-layout.service` | キーボードレイアウト通知。 |
| `argvus-polkit.service` | PolicyKit エージェント統合。 |
| `argvus-session-loading.service` | コンポジタのハンドオフからデスクトップコンポーネントの起動までのセッション読み込みオーバーレイ。 |

`argvus-blueman-applet.service` はセッションパッケージでも出荷されます。一部のユニットはオプションであったり、条件付きで開始されることがあり、すべてのセッションでアクティブになるわけではありません。

ユニットのソースチェックアウトだけでなく、有効な定義を確認するには:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
システムレベルの`argvus-firewall.service`は`argvus-firewall`に属します。カレンダーパッケージは`argvus-taskbar-calendar.service`をユーザー単位として個別に出荷します。`argvus-clock.service`は`argvus`パッケージのユーザー単位であり、`WantedBy=graphical-session.target`はARGVUSクロック設定から有効にされます。

## セッション管理コマンド

`argvus sessionctl`コマンドはセッションのライフサイクルを管理し、ログアウトせずに特定のコンポーネントを再起動できます:

### セッションライフサイクル

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### コンポーネント別の再起動

セッション全体に影響を与えずに個別のサービスを再起動する:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### 環境の同期

環境変数をセッションにインポートする:

```sh
argvus sessionctl import-environment
```
これは、Wayland、Hyprland、Qt、カーソル、およびDBusの変数をシェルとsystemdのユーザーサービス間で同期します。

## サービスユニットの参照

追加のユーザー・ユニットは、`systemctl --user`を使用して直接管理できます:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
初回セッションブートストラップのログは次の場所に保存されます:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
