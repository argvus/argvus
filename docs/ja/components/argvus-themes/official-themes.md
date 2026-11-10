---
title: 公式テーマ
description: 公式のARGVUSテーマカタログ、デフォルトのハイライトカラー、テーマの選択方法とそのStickyモードまたはSpacedモードについて。
---
このページでは、公式テーマパッケージとして利用可能なテーマの一覧を示し、テーマの選択方法について説明します。インストール方法については、[テーマのインストール](/docs/components/argvus-themes/installing-themes/)]を参照してください。

## テーマを選択する

1. **コントロールセンター → 外観 → テーマ**を開きます。
2. **公式テーマ**を選択します。
3. **ダーク**または**ライト**を選択します。
4. テーマファミリーを選択します。テーマは選択するとすぐに適用されます。

選択したテーマはデスクトップ全体に一度に適用されます：実行中のタスクバー、パネル、通知、ウィンドウ、壁紙が、ロード画面や再起動なしでその場で色を変更します。テーマが独自の壁紙関連付けで適用されない限り、ARGVUSは壁紙の選択を保持します。

また、`SUPER + SHIFT + T`を押して、キーボードからテーマセレクターを開くこともできます。矢印キーでカテゴリ間を移動し、`Enter`で選択、`Esc`で閉じます。

## スティッキーとスペースド

各テーマは2つのモードで動作します。モードはテーマに依存しません：テーマを変更しても現在のモードは維持され、モードを変更しても現在のテーマは維持されます。

| モード | ウィンドウ間の隙間 | 外側の隙間 | ウィンドウの角丸 | タスクバーとシェルの余白 |
| --- | ---: | ---: | ---: | ---: |
| スティッキー | `2` | `0` | `0` | 端に近い |
| スペースド | `10` | `18` | `4` | `18` |

モードを切り替えるには、**コントロールセンター → 外観 → モード**を開き、**スティッキー**または**スペースド**を選択してください。これらのレイアウト値に対する手動調整は、次回の明示的なテーマ選択まで有効です。

## ダークテーマ

| テーマ | パッケージ | デフォルトハイライト |
| --- | --- | --- |
| ワンダーク | `argvus-theme-one-dark` | `#61AFEF` |
| ドラクラ | `argvus-theme-dracula` | `#BD93F9` |
| シルバーダーク | `argvus-theme-silver-dark` | `#595959` |
| スレートダーク | `argvus-theme-slate-dark` | `#7391A5` |
| ユニバース | `argvus-theme-universe` | `#EEEEEE` |
| グルーヴボックスハイダーク | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| グルーヴボックスダーク | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| ロゼパイン | `argvus-theme-rose-pine` | `#C4A7E7` |
| 東京ナイト | `argvus-theme-tokyo-night` | `#7AA2F7` |
| ソリチュード | `argvus-theme-solitude` | `#798186` |
| サンセット | `argvus-theme-sunset` | `#E2BE8A` |
| マトリックス | `argvus-theme-matrix` | `#82FB9C` |
| モノカイダーク | `argvus-theme-monokai-dark` | `#78DCE8` |
| レディミスト | `argvus-theme-lady-mist` | `#354B78` |

## ライトテーマ

| テーマ | パッケージ | デフォルトハイライト |
| --- | --- | --- |
| GitHub Light | `argvus-theme-github-light` | `#0969DA` |
| Solarized Light | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Light | `argvus-theme-nord-light` | `#5E81AC` |
| One Light | `argvus-theme-one-light` | `#4078F2` |
| Everforest Light | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Light | `argvus-theme-gruvbox-light` | `#458588` |
| ウォーターカラー | `argvus-theme-watercolor` | `#1A639C` |
| Obsidian Light | `argvus-theme-obsidian-light` | `#334A78` |

## 組み込みテーマ

**ARGVUS Dark** と **ARGVUS Light** は `argvus-appearance` によって提供され、**公式テーマ** の下で常に利用可能です。ARGVUS Dark はデフォルトハイライト `#3590BD` を使用し、ARGVUS Light は `#181818` を使用します。

## テーマが制御するもの

テーマは、デスクトップコンポーネントで使用されるパレットと表面処理、デフォルトのハイライトカラー、デフォルトの壁紙の関連付け、透明度およびぼかしのデフォルトを定義します。デスクトップコンポーネントは自分自身のコピーを保存するのではなく、この状態を読み取るため、テーマの変更はコンポジター、タスクバー、コントロールパネル、ロック画面、通知、およびアプリケーションに同時に反映されます。

デフォルトのハイライトカラーはあくまで出発点に過ぎません。任意の6桁のカラーに置き換えることができ、[ハイライトカラーのカスタマイズ](/docs/components/argvus-themes/highlight-color/)]を参照できます。
