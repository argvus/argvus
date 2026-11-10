---
title: テーマと壁紙のトラブルシューティング
description: テーマの伝播と生成された外観ファイルを診断する。
---
$XDG_CONFIG_HOME/argvus/config.json の `appearance` 正規セクションを確認し、次に  `$XDG_CONFIG_HOME/argvus/config.json` `$XDG_CONFIG_HOME/argvus/data/generated` の下に生成されたファイルを検査してください。レガシーのドットファイルは、移行用の入力にのみ使用されます。

data/generated/ 以下で唯一のライターは `argvus-config` です。壊れたテーマや壁紙を復元するには、正規のドキュメントから全体の生成を再構築し、その結果を生成された消費者を直接編集するのではなく適用してください。

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
 `data/generated/` `argvus theme-switch`（`argvus config theme-apply`を実行する）および`argvus config accent`はオーケストレーターです：これらは検証、ロック、スプラッシュ表示を行い、`argvus-config`を通じてコミットし、`config.json`を読み取ることができない外部の利用者と調整します。これらは`data/generated/`を書き込むことはなく、テーマトランザクションは別のスクリプトを通じてGTK、ストレージ、または`mode.css`ファイルを書き込みません。アクセント調整のみを再実行するには`argvus config accent --apply`を使用してください。

公式テーマの変更は、保存された外観の値を適用し、影響を受けるランタイムの利用者を更新します。カスタムテーマを適用すると、タスクバー、Hyprlandの再読み込み、壁紙、コントロールパネルのサービスが再起動されます。StickyやSpacedを切り替えると、そのモードの間隔、境界線、タスクバーのマージンが再適用され、その後タスクバー、テレメトリ、Hyprlandの再読み込みサービスが再起動されます。Dunstの通知の色は、通知オフ（Do Not Disturb）が有効な間は例外です：通知オフが解除されるまで、それらの再読み込みは保留されます。別のサーフェスが古い色のままの場合は、テーマビルドが有効であり、選択したテーマであることを確認してから、再度再読み込みを実行してください:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
すべてのテーマ付きファイルはビルドから来ます：インストールされていないテーマ（ `theme.toml` `/usr/share/argvus/themes/` または `~/.local/share/argvus/themes/` の下に theme.toml がない場合）やレンダーできないテーマは拒否され、前のテーマが維持されます。`argvus theme check DIRECTORY` は各レンダーエラーのテンプレートと行を表示します。Waybar バーは JSONC の `reload_style_on_change` がある場合にのみ色をリロードします。スイッチが固まっているように見える場合は、それがまだ実行中かどうかを確認してください；それは `theme-switch.active` に、ロックの横に記録されます:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
マーカー内のまだ生きている pid は、スイッチが完了していないことを意味します。スイッチはテーマロックとプロジェクションに対して最大 60 秒、リロードコマンドに対して 5 秒待機します。プロセスが終了したマーカーは古くなり、次のスイッチでは無視されます。問題を報告する際には、上記のコマンドの出力を含めてください。

プリログイングリーターだけが白色である場合やフォールバックテーマを使用している場合は、プライベートソースとそのパブリックプロジェクションを比較してください。

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
greetdの設定を変更せずにプロジェクションを修復する:

```sh
sudo argvus greeter-setup --sync-themes
```
そのファイルは外観オーケストレーターによる事前認証のために公開されており、`data/generated/` の外部に存在するため、単純な `argvus config apply` では更新されません。

挨拶プログラムが Kitty キャッシュで `Invalid color name: 'null'` を報告する場合は、`argvus terminal` パッケージを更新し、挨拶プログラムを再度開いてキャッシュを再生成してください。標準のアクセントが存在しない場合はデフォルトであり、Kitty の色ではありません。現在のターミナルランチャーは、`null` の JSON および無効な古い値を拒否し、`active_tab_foreground` に書き込む前にそれをチェックします。

複数のテーマ変更後にデフォルトのターミナルが開かなくなった場合は、手動で編集するのではなく、派生した Kitty キャッシュを確認してください:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
上記のコマンドは標準的な設定の優先度を読み取ります。`.active-theme` は互換性のためのプロジェクションに過ぎません。キャッシュは `argvus terminal` によって再生成され、テーマの適用と同期されます。`/usr/lib/argvus/loading-theme/splash` が存在しない場合や古い場合は、パッケージの更新が不完全であることを示します。ライブセッションの結果を判断する前に、関連パッケージを再構築してインストールしてください。

ウィンドウの枠が前のテーマカラーを保持している場合、Hyprland の Lua ファイルを手動で編集しないでください。テーマやアクセントの変更は `config.json` から投影され、その後 `argvus sessionctl reload` が Hyprland をリロードし、セッションのライフサイクルを通じてアクティブ、非アクティブ、グループの枠色を再適用します。

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
境界線の*設定*は、コントロールセンターおよびコントロールパネルの`argvus config borders`（またはその`borders-switch.sh`代理）を通じて変更されます；リロードはセッションのライフサイクルを通じて確定された色を再適用し、再導出は行いません。

計画と実際の合成器の値を確認してください：

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
もし`hyprctl`が現在のHyprlandソケットに到達できない場合、ボーダー投影は失敗として報告され、依存するコンポーネントは正常にリロードされたとは見なされません。再試行する前にユーザーセッションのジャーナルを確認してください。
