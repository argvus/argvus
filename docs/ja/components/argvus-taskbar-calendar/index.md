---
title: カレンダー
description: ARGVUS タスクバーのカレンダーを使用してください。
---
`argvus-taskbar-calendar` は、Waybar の日付モジュールから、または次の方法で開かれる GTK4 カレンダーポップアップです:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
ローカルイベント、リマインダー、およびICSのインポート/エクスポートをサポートしています。ユーザーサービスは`argvus-taskbar-calendar.service`です。インストールされているバージョンでサポートされるオプションには`argvus taskbar-calendar --help`を使用してください。

カレンダーは、StickyおよびSpacedバリアントを含む、アクティブなARGVUSテーマに従います。テーマの変更は生成されたユーザーキャッシュを通じて適用されます。実行中のポップアップに明示的な更新が必要な場合は`argvus taskbar-calendar reload`を使用してください。

タスクバーデートモジュールから開く場合、ポップアップはバーの端にアンカーされます。上部のタスクバーの場合は下方向に、下部のタスクバーの場合は上方向に開き、日付モジュールの右端に揃えられます。タスクバーのジオメトリを使用しない直接起動の場合は、ポインターまたはモニタのフォールバックが使用されます。

## コマンド

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## イベントとリマインダー

イベントはデフォルトではオフになっています。ポップアップの **設定 > イベント** でオンにしてください。選択内容は状態ディレクトリに保存され、リマインダーを表示する `argvus-taskbar-calendar.service` も有効化および起動されます。オフにするとサービスは停止します。

- **+ EVENT**（または `n`）は選択した日の編集画面を開きます。今日からイベントを作成・変更できます。過去の日のイベントも開いて削除することは可能です。
- 新しいイベントは09:00に開始し、設定で指定されたデフォルトの長さで継続します。開始時刻を変更しても長さは維持されます。終了時刻が開始時刻より前の場合、そのイベントは翌日に終了します。
- すべてのイベントは、**リマインダー** と **繰り返し** ボックスの内容に関わらず、開始時に通知とサウンドが表示されます。開始後に保存したイベント（たとえば、その日に作成された終日イベント）は、30秒以内に通知されます。
- **リマインダー** は、開始前に設定した時間で前もって通知を追加します（その時間がすでに過ぎている場合は開始時に通知されます）。
- 終日イベントの場合、時間フィールドは **通知時間 (NOTIFY AT)** になります：イベントは1日全体をカバーし、その時間に通知されます（00:00は日の開始時に通知されます）。イベントを再度開いたときも時間は保持されます。
- 終日イベントの場合、**繰り返し (Repeat)** はその時間の後、設定された間隔で再度通知されます。
- 開始時間を変更せずにイベントを編集しても、すでに表示されたリマインダーは繰り返されません。
- 通知はカレンダーの言語（設定 > 言語）に従います。
- イベントがある日にはドット (`•`) が表示されます。これは、一晩続くイベントや複数日イベントの全ての日を含みます。日付は現地のタイムゾーンに従います。
- ローカルイベントは終了後10分で自動的に削除されます。保存時にすでにイベントが終了している場合でも、保存後少なくとも10分間は利用可能です。

インポートされた`.ics`ファイルは、日付、タイムゾーン（`TZID`）、およびアラーム（`VALARM`）を保持します。インポートされたイベントも、自身のアラームに加えて開始時に通知されます。繰り返しイベント（`RRULE`）は保存およびエクスポートされますが、最初の発生のみが表示および通知されます。パッケージのアップデート後、リマインダーサービスは終了し、systemdが30秒以内に新しいバージョンを起動します。`sync`は設定されたリモートカレンダーのみを報告します。CalDAVの同期はまだ実装されていません。

## パス

- 設定: `/etc/argvus/taskbar/calendar/config.toml`（システム全体; パッケージにデフォルトで付属）。
- スタイル: `/etc/argvus/taskbar/calendar/style.css` および `theme.css`; アクティブな ARGVUS テーマの色は、以下のキャッシュを通じてビルドされたテーマから取得されます。
- データ: `$XDG_DATA_HOME/argvus-taskbar-calendar/`（または `~/.local/share/argvus-taskbar-calendar/`）。
- 状態: `$XDG_STATE_HOME/argvus/taskbar/calendar/`（または `~/.local/state/...`）；イベントトグルはそこに保持されます。
- キャッシュ: `$XDG_CACHE_HOME/argvus-taskbar-calendar/`（または `~/.cache/argvus-taskbar-calendar/`）。

ハイライト色は `~/.config/argvus/data/.accent-color` から読み取られ、テーマの色の上にランタイムで適用されます。

自動言語選択はユーザー `locale.conf` に従い、デフォルトは英語です。明示的なカレンダーの言語設定は、自動選択より優先されます。

タスクバーのカレンダー、ARGVUSフォーカス、ARGVUSクロック、コントロールパネルは相互排他です：一方を開くと他は閉じられます。

カレンダーの読み取り失敗は、空の予定表とは別に表示されます。データベースのオープン、日次イベントの読み込み、月次指標の読み込みには、それぞれ異なるメッセージが表示されます。再試行するには、再度日を選択してください。更新が失敗した場合、以前に読み込まれたデータは同じ日または月に限り保持されます；他のクエリからの有効な結果は引き続き利用可能です。

## 共通言語カタログ

インターフェイスは、すべての10言語で共通の`argvus-i18n`カタログを使用します。テキストはマスターワークスペースの`resources/locales/<locale>/taskbar-calendar.json`に保存され、`argvus`によってインストールされます。英語がフォールバック言語です；新しい言語ではもはやローカルの二言語辞書は必要ありません。
