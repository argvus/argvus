---
title: ARGVUS設定
description: ARGVUSによって使用される論理的なJSON構成契約。
---
ARGVUSは、管理されたユーザー設定を`$XDG_CONFIG_HOME/argvus/config/`（通常は`~/.config/argvus/config/`）の下にあるモジュール化されたJSONファイルで保持します。これらはARGVUSによって管理される設定の標準的な論理モデルを形成し、`argvus-config`は`data/generated/`の下にファイルを書き込む唯一のコンポーネントです。ネイティブファイルおよび生成されたファイルは、互換性の投影またはアプリケーション固有のオーバーライドとして残ります。

## ユーザーガイド: クイックスタート

ほとんどのユーザーにとって、設定はARGVUSコントロールセンターまたはこのコマンドラインツールを通じて行われます:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
管理者および高度な設定については、以下を読み続けてください。

ユーザー root には、標準のドキュメントと管理されたデータのみが含まれます:

```text
~/.config/argvus/
├── config.json
└── data/
```
ファイルは原子単位で書き込まれ、ユーザー専用に保持され、`.config.lock`によって保護されます。`data/`ツリーの外側にあり、投影中に入れ替えることができます。以前のバージョンは`data/backups/config.json.bak`に保存されています。

## コマンドライン

`argvus config`サブコマンドは、`argvus`パッケージによってプロセス内で提供されます:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure` はスキーマのデフォルトで新しい `config.json` を生成するか、明示的な値を置き換えずに不足しているフィールドを埋めます。JSON が不正な場合は失敗します。`migrate` は既存のレガシー外観状態をインポートしますが、`config.json` に既に存在する値は置き換えません。`argvus sessionctl` は、設定変更後のリロードを適用する前に `recover`、`ensure`、`migrate` および `project` を実行します:

```sh
argvus sessionctl apply-config
```
`apply`および`rebuild`は、`config.json`からすべてのプロジェクションを再構築するためのエイリアスです。`project`は従来の綴りであり、`--force`を受け入れます。`apply`は、コミット後のランタイム再読み込みを`argvus sessionctl`に委譲します。準備や単独のテストには`ARGVUS_NO_RUNTIME=1`を設定します。生成はステージングで行われ、`data/`ツリーはすべてのプロジェクターが成功した後にのみ入れ替えられます。したがって、`data/generated/`を削除して`argvus config apply`を実行することは、HyprlandやTaskbarの出力の部分的なシーケンスを公開せずに再現可能な再構築です。このコマンドは派生セクションのハッシュとランタイムの判断を`$XDG_STATE_HOME/argvus/config-projection.json`（通常は`~/.local/state/argvus/config-projection.json`）に記録します。マニフェストも生成されたファイルも正しいソースではありません。`apply`は、`argvus sessionctl`を通じてコミット後の再読み込みを実行します。`project`は互換性のためにプロジェクション専用のままです。

すべての永続的な変更は同じルールを使用します：標準文書が変更された場合、CLIはコミット後に中央集権的な`argvus sessionctl reload`を呼び出します。リロード結果は`data/internal/last-reload.json`に記録されます。

したがって、`argvus sessionctl reload`は*適用*ステップであり、投影ステップではありません。受信マニフェストが保留中の作業を報告する場合、計画で記述された対象のコンシューマを復元し、`argvus config project`を再度呼び出すことはありません；再投影すると同一のツリーが再構築され、変更が生成した計画が破棄されます。コミットされた計画が存在しない場合のみ、例えばセッション起動時などに、マイグレーションと投影にフォールバックします。

### リロード対象

`argvus config session-plan` は、変更されたセクションのターゲットでリロードに応答し、`argvus sessionctl` はそれらのみを実行します。スプラッシュは、ターゲットがサーフェスを再作成する場合にのみリロードをカバーします: Waybar バーまたは画面に戻るコントロールパネルはすべてのウィンドウをシフトさせます。それ以外のすべてはその場でリロードされます。

| 変更 | 対象 | スプラッシュ |
| --- | --- | --- |
| テーマ、ハイライトカラー、またはGTKモード | `hyprland`, `consumers`, `wallpaper`, `polkit` | なし |
| 壁紙のみ | `wallpaper`（hyprpaper IPC; swaybg専用でサービスが再起動） | なし |
| `effects`（ぼかし、アニメーション、透明、影） | `hyprland`, `consumers` | なし |
| ウィンドウの`layout`（ギャップ、枠線） | `hyprland` | なし |
| `layout.taskbar`（位置、余白） | `hyprland`, `taskbar`, `widget_telemetry` | はい |
| `fonts` | `consumers` | なし |
| `control_panel`（スイッチ、カード） | `control_panel` | はい |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | なし |
| `power` | `hypridle` | なし |

`consumers` は `argvus config theme-runtime consumers` を実行します（バーのスタイルシート、IPC 経由のコントロールパネル、Hyprland の色、Dunst のテーマ再読み込み、btop および kitty）。Dunst は「おやすみモード」が有効な間はテーマの再読み込みを延期します。`wallpaper` は `argvus config theme-runtime wallpaper` を実行し、そこで有効な `/appearance/wallpaper` を表示します。`layout.taskbar` のハッシュは、ウィンドウレイアウトとは別にプロジェクションマニフェストに保持されるため、ギャップの変更によってバーは再起動されません。コミットされたプランが存在しない場合に使用されるフォールバック `full` プランは、スプラッシュ下のすべてのサーフェスを再起動します。

各有効な構成には運用世代があります。それは `data/internal/generation.json` に保存され、`$XDG_STATE_HOME/argvus/config-projection.json` の下のプロジェクションマニフェストに記録されます。dotfiles の構成の一部ではありません。変更されていない有効な構成を再適用すると、世代は保持されます。`status` はマニフェストハッシュ、世代メタデータ、およびすべての生成ファイルが一致するかどうかを報告します。`apply` は古くなったり不完全なプロジェクションを修復します。

`explain /path` はリゾルバを直接読み取り、有効値、ソース、テーマ、バリアントを出力します。生成されたファイルを解析したり、二番目の優先ルールを作成したりすることはありません。

テーマとエフェクトのプロジェクションは、`argvus-config`自体で実装されています。
テーマトランザクション（`argvus theme-switch` は `theme-apply` を実行します）、`argvus config accent` および `argvus effects-toggle` はプロジェクションを行いません：
それらはこのCLIを通じて検証、ロック、コミットし、その後 `config.json` を読み取れない外部コンシューマーを調整します。`argvus effects-toggle` は出力を持たない薄いデリゲートです。アピアランスヘルパーはプロジェクターから呼び出されることはなく、`data/generated/` 以下に書き込むことは決して許されません。

### プロジェクションサーフェス

`project_effective` は、公開前にすべての ARGVUS 所有コンシューマーを1つのステージングツリーにレンダリングします。

| サーフェス | 出力 |
| --- | --- |
| Hyprland | `generated/hypr/`（テーマプロファイルを含む）、`spaces-effective.conf`、`borders-effective.conf`、`hypridle.conf`、`monitors.lua` |
| レイアウト/エフェクト | `generated/effects/`、`generated/spaces.conf`、`generated/borders.conf` |
| Waybar | `generated/waybar/`、`argvus-taskbar.css` および `argvus-widget-telemetry.{jsonc,css}` を含む |
| ウィジェットテレメトリテーマ | `generated/widget-telemetry/` および `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| 通知 | `generated/notifications/` および `dunstrc` |
| ロック画面 | `generated/hypr/hyprlock.conf` および Hyprtoolkit テーマプロファイル |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superfile | `generated/superfile/` |
| ターミナルおよび Qt プロファイル | `generated/terminal/`、`generated/qt6ct/` |
| フォント | `generated/fonts.conf` および Waybar スタイルシート内の管理されたフォントブロック |
| リムーバブルデバイス | `generated/removable-devices/theme.css` |
| 壁紙 | `generated/hypr/hyprpaper.conf` および `data/hypr/hyprpaper.conf` |
| コントロールパネルおよびデフォルト | `data/control-panel/`、`data/control-center/defaults.json` |
| カレンダー、入力、キーボード、電源 | カレンダー TOML、入力フラグメント、キーボード状態、スリープ防止マーカー |

テーマレイヤーは生成されたルートの外側にも反映されています：
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` と
`data/hypr/application-style.conf`。これらのコピーは、外観セクションが変更されるとパッケージ化されたデフォルトから置き換えられ、その後プロジェクターは
自分の区切られた管理ブロックのみを書き換えます — Waybar スタイルシート内のフォントブロックと
テレメトリプロファイル内の `ARGVUS_TELEMETRY_*` ブロック — したがってマーカー外のテキストはテーマスイッチでは保持されません。`fonts.targets` と
`control_panel.widget_telemetry_blocks` は標準のままです：コントロールセンターは
`argvus-config` を通してそれらを維持し、これらのファイル自体は決して書き込みません。

ウィジェットテレメトリーブロックの選択は `/control_panel/widget_telemetry_blocks` で正規化されており、7 つのブロック名の配列（`system`、`cpu_gpu`、`memory`、`storage`、`processes`、`network`、`keys`）です。従来の `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` ファイルは移行のフォールバックとして読み取り専用であり、書き込まれることはありません。` and ` and blocks all は正規配列を単一のアトミック書き込みでパッチし、`apply-state` は呼び出し元によって選択がすでに永続化されていると想定します。

Hyprland のスペーシングおよびボーダーの設定も同じ境界に従います。argvus の config spaces および config borders コマンド（およびそれらの古い互換性ラッパー）は /layout を更新します。Rust は data/generated/spaces-effective.conf と data/generated/borders-effective.conf を生成し、検証とライブ hyprctl の適用を管理します。.spaces と .borders は移行専用の状態であり、真実の情報源となることはありません。

アクセントと GTK モードの状態は同じ Rust 境界を使用します。アクセントの所有権には ` or ` or argvus config accent --theme-default を使用し、GTK モードには `argvus config mode toggle|set dark|light` を使用してください。従来のアクセントおよびモードスクリプトは互換性ラッパーであり、`.accent-color`、`.accent-custom`、`.gtk-mode` は移行／投影用ファイルのみです。

論理的な ARGVUS パスも `argvus config paths` によって解決されます。従来の `paths.sh` ファイルは互換性関数のみを公開し、ネイティブのオーバーライド、ユーザーデータ、生成データ、システムフォールバックは Rust パスモデルによって解決されます。

`audio` はデフォルトの出力先を変更し、その結果を `/audio` に1ステップで記録します：`argvus config audio volume <N|+N|-N> [--limit PERCENT]` と `argvus config audio mute <on|off|toggle>`。PipeWire にはすでに値があるため、設定を再読み込みせずにストアに書き込みます。Waybar のボリュームモジュール、Hyprland のメディアキー、コントロールパネルがこれを使用するため、`argvus config project` は、`/audio` をシンクに投影しても、以前のボリュームを復元することはありません。100%を超えるボリューム（Hyprland のキーでは最大150%まで）は100として保存されます。

`patch` は、1 回のロックされた読み取り-修正-書き込み操作で、そのキーが JSON ポインタである JSON オブジェクトを適用します。`apply-theme` は同等のアトミックなテーマトランザクションを実行し、テーマのオプションの外観プロファイルまたはパッケージ化されたデフォルトから管理される外観セクションを置き換えます。変更コマンドは `NextUserConfig` を構築し、`EffectiveConfig` を解決し、すべてのプロジェクターをステージングし、その後にのみ標準 JSON を公開します。運用上の `.config-transaction.json` マーカーにより、`argvus config recover` はデータと JSON の公開の間のクラッシュ後に前の構成を再投影できます。テーマのデフォルトは各テーマのマニフェスト（`themes/<theme>/theme.toml`）によって宣言されます：デフォルトのアクセント（`palette.accent`）、GTK カラースキームおよび壁紙です。CSS、Lua、Rasi および Qt ファイルは、そこからテーマビルドにレンダリングされます。

テーマ（ライト/ダークを含む）を変更すると、ユーザーがカスタムカラーを選択していた場合でも、ハイライトカラーはテーマのデフォルトにリセットされます：カスタムカラーは次回のテーマ変更まで保持されます。テーマ変更後に`argvus config accent`を使用して再設定してください。`argvus config theme-apply`を使ってテーマを適用すると、マニフェストから`/appearance/accent`と`/appearance/gtk_mode`が書き込まれ、`argvus config accent`は`accent_custom`が設定された状態で選択したカラーを書き込みます。これらは1つのコマンドで行います。

`apply-theme` は変種を明示的に `--variant sticky|spaced` として受け取り、マニフェストを通して要求された名前を解決するため、リクエストが他の変種のアセットに解決されることはありません。`theme-manifest THEME` は同じ解決を読み取り専用で公開します：解決された `asset_id` および `layout_variant` と共にマニフェストのエントリを出力し、これは呼び出し元がパッケージ化されたアセットにアクセスするために必要なものです。Sticky は接尾辞のない ID であり、Spaced のみが `-spaced` の接尾辞を持つため、`dracula-sticky` および `argvus-dark-dracula-sticky` は受け入れられ、`dracula` に正規化されます。

`--raw` を使用すると、文字列の値は JSON の引用符なしで出力され、オブジェクトや配列は JSON 値のままです。テーマやレイアウトのドットファイルはプロジェクションヘルパーによって書き込まれなくなり、`config.json` がこれらの設定の唯一の永続的なソースとして残ります。プロジェクションに失敗した場合、セッションのリロードはデスクトップサーフェスの再起動前に停止するため、古い部分的なテーマが現在のものとして提示されません。

## エクスポートとインポート

プロファイル交換には同じ論理モデルが使用されます:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
完全な管理デスクトップモデルには`--scope desktop`を使用してください。欠落しているフィールドはARGVUSのデフォルトを使用してください。生成された出力は直接編集しないでください。

コントロールセンターのテーマのエクスポートとインポートでは、テーマとそのビジュアルプロファイルを交換します。アーカイブにはスキーマ2のテーマディレクトリ（`<id>/theme.toml`、オプションで`appearance.json`および`effects.conf`、`overrides/`、壁紙）が含まれており、`$XDG_DATA_HOME/argvus/themes/`の下にインストールされます。プロファイルには外観、レイアウトジオメトリ、ビジュアル効果、フォント、パネルの表示が含まれます。オーディオ、キーボード、ネットワークおよびその他の非ビジュアルの設定はテーマには含まれません。完全な外観設定を移動するには上記の`appearance`スコープを使用し、完全な設定バックアップには`desktop`スコープを使用してください。

現在のスキーマでは、設定は`appearance`、`layout`、`effects`、`fonts`、`control_panel`、`default_apps`、`keyboard_shortcuts`、`hyprland`、`displays`、`power`、`session`、`calendar`、および`removable_devices`の下に分類されています。`default_apps`は好みのアプリケーションの唯一の永続的なソースであり、`keyboard_shortcuts`は文字列値を持つ安定した英語のロケールに依存しないキー、または無効化されたバインディングの場合には`null`を格納します。

コントロールセンターは、外観ページを開くときに標準的な外観値を読み取り、`argvus-config`を通じて変更を書き込みます。既存のネイティブ状態ファイルは、外観スクリプトおよび旧バージョンのインストール用の投影として残ります。

コントロールパネルは、カードの順序と表示に`control_panel.cards`を使用し、パネルのマスタースイッチには`control_panel.enabled`を使用します。`control-panel/cards.json`および`state/control-panel`ファイルは互換性のための投影として保持されます。したがって、コントロールパネルで行われた変更は`desktop`のエクスポート範囲の一部となります。

フォントの設定は、タスクバー、テレメトリ、コントロールパネル、システム、アプリケーション、端末、およびブラウザのターゲットのフォントファミリー、スタイル、サイズに`fonts.targets`を使用します。`fonts.rendering`は、アンチエイリアス、ヒンティング、サブピクセル順序、DPIの設定を保存します。`fonts.conf`、フォント設定ルール、デスクトップ設定、およびWaybarのスタイルシート内の管理されたフォントブロックは投影です：コントロールセンターは`/fonts/targets/*`から`argvus-config`までを保持し、プロジェクターはその区切られたブロックのみを書き換えます。

Hyprland の設定では、ショートカットに `keyboard_shortcuts`、マウスとタッチパッドの設定に `hyprland.input`、XKB レイアウト、バリアント、およびオプションに `hyprland.keyboard` を使用します。マニフェストのバインディング ID は安定した `config_key` にマッピングされます（例えば `window.close` → `close_window` および `window.drag_mouse` → `drag_window__floating_window_only`）。ディスプレイ設定では `displays.monitors`、`displays.workspaces`、および `displays.primary_monitor` を使用します。`data/generated/` 下のネイティブ TOML ファイルと生成された Lua ファイルは、Hyprland によって使用される投影のままです。

すべての `effects` スイッチはデフォルトで有効になっています（アニメーション、ぼかし、ウィジェットのテレメトリ、および各透明表面）、仮想マシンや低電力セッションでも同様です。ウィンドウシャドウ（`shadow_enabled`）および揺れ（`wobble_enabled`）はデフォルトでオフです。エフェクトの設定は、アニメーションのマスタースイッチ（`effects`、`animations`、`hyprland.lua` にプロジェクトされた `hypr/state.lua` に適用され、エフェクトスイッチである `blur_global_enabled` とは独立しており、 `transparency_<surface>_enabled` `taskbar`、`control-panel`、`widget-telemetry`、`terminal`、`launchers` を含む transparency_<surface>_enabled と組み合わせ、いずれかが有効の場合は有効として読み取られます; `argvus effects-toggle animations|effects` で切り替え可能）、グローバルぼかしスイッチ（`blur_global_enabled`）、Hyprland のぼかしパラメータ（`blur_size`、`blur_passes`、`blur_brightness`、`blur_noise`、`blur_contrast`、`blur_vibrancy`、`blur_vibrancy_darkness`）、ウィンドウシャドウ（`shadow_enabled`、`shadow_sharp`、`shadow_color`）を使用します。 `shadow_range`、`shadow_render_power`、`shadow_offset_x`、`shadow_offset_y`、`shadow_scale`）、アニメーションの値（各アニメーションについて`animation_<leaf>_speed`、`_curve`、`_style`、`_popin`および`_enabled`、カスタムカーブ用の`animation_bezier_x1`から`animation_bezier_y2`および`animation_spring_mass`、`animation_spring_stiffness`、`animation_spring_dampening`）およびサーフェスごとの透明度の値。エフェクトはテーマ所有であり、バリアント所有ではない：StickyとSpacedは同じ`<theme>.conf`のエフェクト投影を使用するが、レイアウト投影は異なる。現在のサーフェスには`taskbar`、`control-panel`、および`widget-telemetry`が含まれる；透明度の値は`0`から`100`までのパーセンテージ（コントロールセンターとランチャーのサーフェスは`15`がデフォルト、それ以外は`50`がデフォルト）、ブラーのパラメータは`schema.json`で宣言された範囲内の数値、Enabled状態はブール値。アニメーションの値は`argvus-config`によっても検証される：速度は `1` から `10` までの数値 (小数も許可)、ポップイン パーセンテージは `1` から `100` までの整数、カーブはアニメーション ページの名前である必要があり、スタイルはアニメーション ページの名前であり、アニメーション自体のリーフで受け入れられる必要があります (たとえば、`popin` は拒否されます) `animation_workspace_in_style`、`gnomed` はウィンドウの開閉にのみ受け入れられ、スライド バリアントはウィンドウとレイヤーの葉で拒否されます)、ベジェ Y ポイントは `-1` と `2` の間に留まり、Hyprland 0.56.2 が解析する範囲、およびその他の曲線パラメータは文書化された範囲内に留まります。 `layers` および `switching` リーフはもう存在しないため、削除前に保存された設定はロード時に `animation_layers_enabled`、`animation_layers_speed`、`animation_layers_curve`、および `animation_switching_*` キーを失います。現在のセット外のスタイルは削除され、`-1` から `2` の外にあるベジェ Y ポイントはクランプされるため、パッケージ化されたデフォルトが再度適用されます。 削除されたもの。 

電源の設定には`power.screen_off_minutes`、`power.lock_minutes`、`power.keep_awake`、および蓋/電源ボタンのポリシーが使用されます。予測される形式は`data/generated/hypr/hypridle.conf`で、マーカーは`data/power/keep-awake`です；`.keep-awake`は移行入力専用です。`session.language`は廃止されています：既存の値は無視され、新しい書き込みは拒否され、言語互換ファイルは投影されません。代わりにユーザーlocale.confで`LANG`を設定してください。カレンダー設定には `locale.conf` `calendar`が使用され、タスクバーのカレンダーユーザーTOMLに投影されます。リムーバブルデバイスの設定には`removable_devices`が使用されます；リムーバブルデバイスアプリケーションでは、このセクションをパッケージ済みデフォルトに対するユーザーオーバーライドとして扱います。

`migrate`は、既存の正規値を保持しながら、古いルートファイルとセクションを一度インポートします。レガシーの`defaults.json`、`/defaults`、`/hyprland/keybindings`、`keybindings.toml`、ドットファイル、`generated/`およびコンポーネントディレクトリは、`data/`の下に移動されます；再ロードを繰り返しても古いルートレイアウトは再作成されません。現在のテーマを再度選択することは冪等です：もし正規状態と投影がすでに一致していれば、投影プランは変更されたセクションがないと報告し、セッションはスプラッシュとサービスの再起動をスキップします。

Powerも同じ投影契約に従います：コントロールセンターは`/power`を`config.json`に書き込み、`argvus config project`は`data/hypr/hypridle.conf`を原子単位でレンダリングし、その後でのみ`argvus sessionctl`が`argvus-hypridle.service`を再起動できます。そのファイルを削除することは`argvus config project`で復旧可能です；`0`の値は対応するタイムアウトを省略し、「決して」を意味します。

ロックスクリーンの外観は、Rustによってカノニカルテーマ、アクセント、エフェクト、壁紙、およびフォントの状態から`data/hypr/hyprlock.conf`に投影されます。歴史的な`hyprlock-theme.sh`のエントリポイントは互換性のみであり、`argvus config lock apply`に委譲されます。`--invalidate`は派生したぼかし壁紙キャッシュを無効化するだけです。

電源タイムアウトの変更は`/power/lock_minutes`および`/power/screen_off_minutes`を真実の情報源として使用します。`argvus config project`は既存のHypridleレンダラーを再利用し、`idle-timeout.sh`のエントリポイントは互換性アダプターのみです。サービスの再起動は引き続き`argvus sessionctl`に委譲されます。

通知の外観はカノニカルテーマ、アクセント、およびエフェクトから`data/notifications/dunstrc`に派生します。歴史的な通知`theme.sh`ファイルは、`argvus config notifications`を呼び出す互換性関数のみを公開しており、Dunstの設定を解析したり変更したりすることはありません。

ディスプレイ検出やBluetooth制御などのハードウェア指向アダプタは現在、Rustディスプレイドメインによってインポートされています：`nwg-displays`は外部インポーターであり、`/displays`と`data/generated/hypr/monitors.lua`はそれぞれ正規および派生の状態のままです。ディスプレイツールは`argvus config monitor-set`、`monitor-primary`、`monitor-workspaces`、および`monitor-remove`でレイアウトを変更し、ディスプレイコマンドは`argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`です。Hyprlandファイルは`/displays`から投影されており、もはやシェルスクリプトはそれを書きません。壁紙選択、タスクバーのユーティリティグループモード、およびデフォルトアプリの検索も`argvus-config` APIを使用しており、それらの歴史的なシェルエントリポイントはUIまたは互換性ラッパーです。Keep Awakeは`/power/keep_awake`の永続的な設定で、`data/power/keep-awake`マーカーに投影され、lock-DPMSはセッションのランタイム状態下にあります。
`$XDG_CACHE_HOME/argvus/lock-dpms`。キーボードレイアウトは現在Rust `argvus-keyboard-layout`ヘルパーによって管理されています：設定された順序は生成されたHyprland入力プロジェクションまたはロケールフォールバックに由来し、アクティブなレイアウトはHyprlandから発見されたセッションランタイム状態のままです。サイクルのエントリーポイントとキーボードレイアウトデーモンは互換性ラッパーに過ぎません；ユーザーサービスはRustデーモンを直接実行し、`argvus sessionctl keyboard-layout cycle`は呼び出し元のための安定したコマンドです。

Bluetoothのランタイム状態は外部アダプタの境界に従いますが、`argvus-config`の一部ではありません：`argvus bluetooth`は`bluetoothctl`周りの型付きパースと検証、制限付きリトライ、システムサービスのチェック、およびRFKill状態を管理します。`argvus bluetoothctl`がエントリーポイントです。アダプタの電源、ペアリングデバイス、接続済みデバイスはカノニカルJSON状態ではなくBlueZのランタイム状態に留まります。ハードウェアのライブBluetooth検証は環境依存です。 
## ランタイムの識別と調整

正確なバイナリ、ビルド識別、設定ルート、生成ルート、アセットルート、マニフェストソースおよびアクティブ世代を確認するにはargvus config status --verboseを使用してください。`argvus config status --verbose`は`synchronized`、`stale`、`incomplete`、`legacy`および`pending-transaction`の投影状態を報告します。

`argvus config --version` は、ビルド環境によって提供される場合にパッケージのバージョン、ビルドリビジョン、およびプロファイルを報告します。開発テストは、バイナリとアセットを明示的に注入する必要があります。`argvus config` または `~/.config/argvus` を暗黙的に使用してはいけません。`apply` は、`argvus-dark-spaced` のようなレガシーの結合識別子を、射影を再構築する前に安全にテーマとバリアントの別々の値に正規化します。このリリースでは、パッケージ化されたアセットの識別情報はマニフェストに記録されません。

## ナイトライト

**デスクトップ & 入力 > ディスプレイ > ナイトライト** を開くか、`argvus control-center displays night-light` を実行します。`argvus config display night-light` を通じて、色温度、手動での起動、および固定ローカルスケジュールを設定します。コマンド、状態、ランタイム要件については、[ナイトライト](/docs/user-guide/night-light/)] を参照してください。

### ぶれ設定 (Hyprland 0.57+)

`effects.wobble_enabled`、`wobble_stiffness`、`wobble_damping`、`wobble_mass`、`wobble_intensity`、および`wobble_mesh`は直接`hypr/state.lua`に投影されます。
これらを読むには`argvus effects-toggle wobble-settings get`を使用するか、
保存してリロードを要求するには`argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`を使用します。`enabled=disabled`は効果をオフにします。

コントロールセンターはアニメーションのドラフトを共有します：`animations-settings get`も`wobble_*`のエントリを返し、`animations-settings set`はこれらのフルネームを既存のアニメーション引数と共に受け入れます。これにより、ドラフト全体が1つの検証済みパッチに永続化され、1回のリロードが要求されます。例えば:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
グローバル`enabled`はアニメーションを制御します；`wobble_enabled`はその独立した設定です。グローバルアニメーションを無効にすると、Luaでの揺れが抑制されますが、その設定は消去されません。無効な値はパッチ全体を拒否します。キーが欠けている場合は、既存の選択を置き換えずにARGVUSのデフォルトが適用されます。[Appearance](/docs/user-guide/appearance/#window-wobble)]で範囲とデフォルトを確認してください。

### ユーザー作成のキーボードショートカット

`/keyboard_shortcuts`は、公式マニフェストアクション用の既存の文字列またはnull値を保持します。カスタムグローバルキーボードショートカットは、`/hyprland/custom_shortcuts`オブジェクトを使用し、`config/hyprland.json`に保存されます。その安定した`custom_` IDは、説明やロケールに依存しません。各レコードには`description`、`keys`、`enabled`、およびタイプ付き`action`があります：

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
この例は `/hyprland/custom_shortcuts` の値であり、完全な設定文書ではありません。ディスパッチャー名はサポートされている Lua ディスパッチャーファクトリーから取得されます。引数はオプションの JSON オブジェクトまたは `null` であり、決して評価されるコードではありません。カスタムショートカットにはサブマップ、マウスバインディング、または高度なフラグはありません。プロジェクションは `data/generated/hypr/custom-shortcuts.lua` をエスケープされたデータテーブルとして書き込み、最後のレコードが削除された場合には空のテーブルを含みます。フルプロジェクションおよび `project --only keyboard_shortcuts` の両方がそれを生成します。Hyprland は有効になったエントリを `hl.bind` で登録します。オプションのファイルおよび個別の登録失敗は管理され、ログに記録されます。

コントロールセンターは、1つの`argvus config patch`トランザクションを使用してレコードを作成/編集し、確認済みの競合を無効にします。既存の構成サービスを通じて適用する前に保存します。適用に失敗しても、成功した保存は元に戻りません。公式デフォルトを復元してもカスタムレコードは保持され、アクティブなカスタムショートカットと衝突する場合は拒否されます。ユーザーの説明はすべてのロケールおよび生成された実際のチートシートで文字通りのテキストのままです。

## グローバルウィンドウモード

**コントロールセンター → デスクトップ＆入力 → Hyprland → ウィンドウ → モード**を開き、**タイルウィンドウ（デフォルト）** または **フローティングウィンドウ** を選択します。同じセレクターはタスクバーのユーティリティグループのウィンドウモードアイコンからも利用可能です。

この設定はすべてのモニターとワークスペースに適用され、ログアウト後も記憶されます。フローティングは既存のアプリケーションウィンドウと新しく開かれたウィンドウを変換します。タイルはこのポリシーで変換されたウィンドウのみを復元します。すでにフローティングであったダイアログやその他のウィンドウはフローティングのままです。フローティングが有効な状態で明示的にピン留めされたウィンドウも、そのピンを維持するためにフローティングのままです。タイルレイアウトは再計算されますが、以前の正確な分割ツリーは復元されません。

`SUPER + SHIFT + Space`はタイル時に通常のウィンドウごとの動作を維持します。フローティングでは、ウィンドウをタイルに戻すことはありません。パネル、タスクバー、通知はレイヤーシェルのサーフェスであり、影響を受けません。全画面表示や最大化は引き続きHyprlandの通常の動作を使用します。

標準の優先設定は `/hyprland/window_mode` で、`$XDG_CONFIG_HOME/argvus/config/hyprland.json` にあります。その生成された投影は `data/generated/hypr/window-mode.lua` です。編集しないでください。ウィンドウごとの復元レジストリは `$XDG_RUNTIME_DIR/argvus` の下にあり、コンポジターのインスタンスにスコープされ、設定のリロード後も保持されます。新しいセッションでは再利用されません。

アプリケーションが保留中または失敗した場合、インターフェースは保存された優先設定とこれを区別します。再度オプションを選択すると再試行されます。コンポジターがない場合、優先設定は次回のログイン時に保存されます。

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## ウィンドウボタンとアプリケーションバー

フローティングモードでは、通常のアプリケーションウィンドウに **最小化**、**最大化 / 元に戻す**、および **閉じる** を持つ ARGVUS タイトルバーが追加されます。タイトルバーをダブルクリックすると最大化の切り替えが行われます。この追加装飾はタイル表示および全画面表示では非表示になります。自分でコントロールを描画するアプリケーションは両方のバーを表示する場合がありますが、ARGVUS の統一されたコントロールは意図的なものです。

セカンドWaybarは、各モニターのメインタスクバーとは反対の端に表示されます。それぞれのモニターの現在のワークスペースに属するウィンドウのみを一覧表示し、そのワークスペースで最小化されたウィンドウも含まれます。オープンされている特別なワークスペースが優先されます。固定されたウィンドウは常に表示され、非表示のグループタブは追加エントリを作成せず、最小化されたグループは1つの復元エントリを持ちます。アプリケーション内のタブは個別に列挙されません。最小化されたエントリは元のモニター/ワークスペースの関連付けを保持します。非アクティブウィンドウをクリックするとアクティブ化され、アクティブウィンドウをクリックすると最小化され、最小化されたウィンドウをクリックすると復元されます。そのコンテキストメニューには復元、最小化、最大化/復元、閉じるが含まれます。未知のアプリケーションアイコンはウィンドウのシンボルにフォールバックします。エントリをアクティブ化または復元するとポインターの位置は保持されます。セカンダリバーは有効なプライマリCSS（色、フォント、境界線、角丸）を使用し、その高さと反対側の端でのマージンミラーを適用します。

`SUPER + Alt + Up`はメインバーを上に、ウィンドウバーを下に配置します。`SUPER + Alt + Down`はその逆を行います。同じ方向を繰り返すと、位置は変更されません。タイルにはメインバーのみがあります。タイルに戻ると、ウィンドウバーを削除する前にすべての最小化されたウィンドウを復元し、その後、元のタイル/フローティングのポリシーを保持します。

最小化は予約済みの`special:argvus-minimized`ワークスペースを使用します。そのワークスペースを手動で使用しないでください。`argvus windows service`プロセスは、識別情報、元のワークスペース/モニター、ジオメトリ、フルスクリーン/最大化およびピン状態を、`$XDG_RUNTIME_DIR/argvus`の下にあるコンポジタインスタンス固有の実行時レジストリに記録します。記録はサービスの再起動や設定の再読み込みによっても保持されますが、コンポジタセッションを超えることはありません。ウィンドウを閉じると、その記録は削除されます。モニターが消えた場合、復元には利用可能な出力が使用されます。バーが存在しないか復元に失敗した場合、新しい最小化はできません。復元に失敗した場合、記録と復元バーは保持されます。

セッションは`argvus-windows.service`と`argvus-window-bar.service`を所有します。後者はWaybarの子プロセスをTileで休止状態に保ちます。hyprbarsプラグインは`argvus-hyprland`で同梱されており、x86_64上のHyprland 0.57.0 ABI向けにビルドされています。ユーザーのホームディレクトリではコンパイルされていません。コンポジタのABIが更新された場合、プラグインを再ビルドしてください。コマンド: `argvus windows list --json`、`watch --json`、`position top|bottom`、`restore-all`、および `minimize|restore|activate|maximize|close ADDRESS`。ウィンドウアドレスはリストから取得されます。`position top --persist` はメインバーを再起動せずに設定を保存/適用します。構成および実行時のエラーの詳細はジャーナルに書き込まれます。表示されるアクションの失敗も翻訳された通知として表示されます。

**Spaced + Floating** では、ウィンドウは外側のギャップと両方のバーによって確保された領域を尊重します。ボーダーとタイトルを含む境界は、同じウィンドウボーダー内に収まります。ARGVUS コントロールを使った最大化は、マージン付きでこの領域を埋めます。復元すると、サービス再起動後も以前のジオメトリを回復します。フルスクリーンは依然として出力全体を使用します。フローティングウィンドウは重なり合うことがあります; タイリングの内部ギャップはそれらを分離しません。

バーによって確保された内側の余白は、ウィンドウの外側の隙間で二重にカウントされません。対称的な間隔では、バーを入れ替えた後も含めて、ウィンドウとバーの見える距離は上下で等しくなります。

`SUPER + Alt + Space` は、タスクバーとコントロールセンターと同じ設定を使用して、グローバルなタイル/フローティングモードを切り替えます。等価なコマンドは `argvus config window-mode toggle` です。同時実行は直列化されます；切り替えは、アプリケーションが保留中の場合でも保存された設定を使用します。ショートカットはキーボードショートカットで変更または無効にできます；`SUPER + Shift + Space` は、個別ウィンドウの操作を維持します。
