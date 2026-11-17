---
title: ARGVUS 電卓
description: ARGVUS 電卓 — GTK4 / ARGVUS
---

# ARGVUS 電卓

`argvus calculator` · launcher: **ARGVUS 電卓**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **関数電卓**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **ラジアン**: 未選択：度.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **クリア**; Backspace: **一文字削除**; Tab / Shift+Tab.
- **結果をコピー**; **履歴**: 100; **履歴を消去**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## 履歴

終了後も履歴を最大100件保存します。履歴を消すと保存データも削除されます。設定は行を選んで操作します。↑/↓で移動、Enterまたはクリックで実行、Escapeで戻ります。

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

常に中央の小さなフローティングウィンドウ（370 × 420）で開きます。関数モードでは拡大できます。

メイン画面には数式、結果、キーのみを表示します。メニューから関数モード、角度単位、コピー、独立した履歴画面を開けます。戻るまたは Escape で計算に戻り、履歴の結果を選ぶと再利用できます。基本モードに戻すと数式を保持して小さいウィンドウに戻ります。

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

ウィンドウを拡大するとキーも大きくなります。新しく起動するたびに標準サイズで開きます。サブ画面の上部には戻る矢印があります。

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

選択した基本・関数モードはすぐに保存され、次回起動時に復元されます。

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
