---
title: セッション起動トラブルシューティング
description: グリーター、Hyprland、および systemd ユーザーセッションの障害を診断する。
---
走る:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
グリーター、`argvus start`、Hyprland、および関連するユーザーユニットを個別に確認してください。パッケージのビルドが成功しても、インストールされたペイロードやアクティブなユーザーオーバーライドが最新であることを証明するものではありません。

次の順序でチェックを行ってください：

1. グリーターがインストールされたARGVUS greetd設定を使用していることを確認します。
2. セッションログを読み、HyprlandのLua設定が欠落しているか無効でないかを確認します。
3. `systemctl --user status argvus-session.target`で準備完了状態とターゲット状態を確認します。
4. すべてのサービスを一度に再起動するのではなく、失敗しているコンポーネントを検査します。

`argvus-session.target`が非アクティブで、ロードオーバーレイがアクティブな場合は、サービスを再起動する前に投影エラーを確認してください：

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config` は `data/generated/` より下にある唯一のライターであるため、`project` は診断中に世代を再現するための権威ある方法です。コミットされたプランがまだ存在しないため、セッションの起動時にこれが実行されます。`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` に書き込むプランは、後の `argvus sessionctl reload` が適用し、再度投影することはありません。

セッションは、致命的な準備やターゲットの起動が失敗した場合に読み込みオーバーレイを停止します。ビジュアルコンシューマの失敗は、文書化されたフォールバックを使用するか、明示的に報告する必要があります。成功したセッションとして隠してはいけません。

## ログインが遅い

もし`greeter.handoff-takeover-failed`が`handoff.master-acquired`なしで現れた場合、取得エラーについて`handoff.master-unavailable card=<name> errno=<n>`を確認してください。別のコンポジターがマスターのときにオブザーバーが開かれると、そのコンポジターがマスターを解放した後でも、errno 13 (`EACCES`) を返すことがあります。ヘルパーはマスターを暗黙的に取得するために同じGPUを再度開きます；`handoff.master-reopened`はこのリカバリを記録し、そのディスクリプタ上で新しいフレームバッファが準備されます。グリーターは失敗したヘルパー（`greeter.handoff-cancelled`）をキャンセルするため、ユーザーのコンポジターが開始する前に別の待機を追加できません。取得が保留中のまま`release`を送信するセッションは`handoff.release-before-master`を受け取ります：ヘルパーは承認する前に自身のDRMディスクリプタを閉じます。Hyprland/Aquamarineでのグリーターコンポジターのクラッシュは別の発見であり、ヘルパーをキャンセルしても、ドライバーやコンポジターの問題が修正されたことを示すものではありません。

`greeter.vt-switch-returned active=<tty>`は、logindのswitch呼び出しが戻ったときにアクティブなVTを記録します。描画タイムアウトもそのエラーに`active-vt=<tty>`を記録します。これらはスナップショットであり、完了したDRMハンドオーバーの証拠ではありません。

前後の比較を行うには、作業を保存し、同じマシン、テーマ、モニター設定でバージョンごとに5回ログインしてください。次のログアウトの前に、`/run/argvus-greeter/timing.log`、セッションログ、およびブートジャーナルを`/tmp/argvus/`の下にある別のサンプルディレクトリにコピーします。次のグリータ実行時には、タイミングログが置き換えられます。各サンプルにブートIDを記録し、そのブート内のタイムスタンプのみを比較してください。認証から最初のフレームのハンドオフ、セッションラッパーのエントリ、最終Hyprlandの起動、読み込みスピナー停止までを測定します。認証からスピナー停止までの中央値と最悪時間を比較し、黒いフレーム、フラッシュ、またはスタックしたスピナーがないか目視で確認します。デスクトップサーフェスの期間のみが、エンドツーエンドのログイン時間ではありません。失敗および回復テストは、保存されていない作業を含むセッションを終了せずに、使い捨てセッションで繰り返してください。

各起動ステップはセッションログにその所要時間を記録します。最後のログインのステップを、最も遅いものから順に確認するには:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
グリーターからデスクトップへの切り替えのタイミングを計るには、グリーターとハンドオフのマークをジャーナルと一緒に読みます。すべて`CLOCK_MONOTONIC`を使用しているので、整列します:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` は最後のログインのみを保持します：グリーター、DRM ハンドオフサーフェス（`handoff.master-acquired`、`handoff.output-found`、`handoff.first-frame`、`handoff.release-received`、`handoff.replaced-by-compositor`）および `greeter.*` のマークです。`handoff.crtc` の行は、ハンドオフが DRM マスターを待つ間のすべての CRTC 変更を記録します；`fb=0` または `mode_valid=false`、あるいは `handoff.output-found ... framebuffer=0` の行は、グリーターコンポジタがハンドオフを受け取る前に出力をオフにしたことを意味し、そのためモニターは再同期する必要がありました。`greeter.watchdog-stop-requested` は、グリーターセッションがコンポジタを待たずに終了したことを意味し、`greeter.compositor-exit-deferred` は、ハンドオフサーフェスが消えた後にコンポジタに終了するよう要求されたことを意味します；`greeter.compositor-exit-requested` 単独は、即座に終了するよう要求されたことを意味します。ジャーナル内の greetd と logind の行が、グリーターセッションの終了とユーザーセッションの開始の間に約半秒のギャップを示している場合 オープニング時に、グリーターコンポジターのテアダウンはログイン中に実行されました。これは、マークが`compositor-in-foreground=true`を示している場合に予想される現象です：グリーターの端末のハングアップは、遅延終了前にコンポジターを終了させます。これは、グリーターがコンポジターの終了前にVTを離れることができなかった場合に発生します：`greeter.vt-switch-requested`の後に`greeter.handoff-drawing`を探してください；2つ目がない場合は、`greeter.handoff-takeover-failed error=<reason>`が理由を示します。`handoff.page-flip`は、ハンドオフがモードセットなしでグリーターの最後のフレームを置き換えたことを意味します；`handoff.page-flip-failed errno=<n>`の後に`handoff.modeset`が続く場合は、カーネルがページフリップを拒否したことを意味します。`handoff.overlay plane=<id>`および`handoff.retired`は、オーバーレイがセッションスピナーが画面上に表示されるまでHyprlandの最初のフレームを隠していたことを意味します；`handoff.overlay-unavailable`はCRTCに使用可能なオーバーレイプレーンがなかったことを意味し、`handoff.retire-timeout`はセッションスピナーが送信されなかったことを意味します。 `retire`。セッションスピナー（`argvus-greeter-handoff`）は `first layer-shell frame callback` と `stop requested` をジャーナルに書き込みます。

外観の変更、テーマファイルの変更、新しいアクセント、または古い起動スタンプの移行後には、`project.theme-switch` または 0 以外の `prepare.startup-theme` エントリが期待されます。 同一のファイルを再インストールすると、コンテンツベースの起動キャッシュが保持されます。 毎回のログインで表示される場合、マテリアライズされたテーマファイルが欠けている可能性があります。 一度 `argvus theme-switch <theme>` を実行してください。

セッションログは `$XDG_STATE_HOME/argvus/session.log`（通常は `~/.local/state/argvus/session.log`）に保存されます。 ユーザーの Hyprland オーバーライドを変更した後、ログアウトして再度ログインすると、セッションエントリポイントが検証され、再度選択されます。 通常の起動フローについては [Graphical session](/docs/argvus-session/graphical-session/) を参照してください。
