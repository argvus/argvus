---
title: プロジェクト
description: argvus プロジェクトを通じたプロジェクトピッカーとプロジェクトの設定。
---
`argvus projects` はあなたのプロジェクトを一覧表示し、開きます。これは `argvus` パッケージのプロセス中のコマンドであり、`SUPER + O` はピッカーを開きます。

## コマンド

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## プロジェクトとは何か

プロジェクトとは、設定（`/projects/paths`）にリストされているディレクトリ、または`/projects/roots`にリストされているルートのサブディレクトリのことです。最大で9つのプロジェクトがリストされます：`SUPER + CTRL + N`ショートカットの番号です。プロジェクトを開くと、そのHyprlandワークスペースにフォーカスが移ります。プロジェクトを最初に開くと、ARGVUSが端末と`/projects/editor`に設定されたエディタをその中で起動します。

設定は`argvus config`を通じて読み書きされるため、コントロールセンターとシェルの両方で同じリストが見えます。

## プロジェクトワークスペースとタスクバー

プロジェクトは特別なワークスペースではなく、通常の名前付きHyprlandワークスペースを使用します。`SUPER + O`はプロジェクト番号、ディレクトリ、および直接の`SUPER + CTRL + N`ショートカットを示します。両方のエントリーポイントは同じワークスペースにフォーカスします。タスクバーには開いているワークスペースのみが表示され、`Project: name`ラベル、アクティブ状態、およびフルパスとショートカットを含むツールチップが付きます。ワークスペースボタンをクリックするとそこに戻れます。タスクバーはワークスペースのモニターに従います。

正規ディレクトリパスは安定したワークスペースの識別子を決定します。リストの順序を変更してもワークスペースではなくショートカットが変更されます。同じディレクトリ名のプロジェクトは別々のままであり、同じディレクトリへのシンボリックリンクは重複が排除されます。ターミナルやエディタの起動は明示的にプロジェクトのワークスペースをターゲットにします。既存のプロセス/ウィンドウを再利用するエディタは、そのアプリケーション自身のワークスペース動作を保持する場合があります。

更新前に開かれたワークスペースは、既存のウィンドウとネイティブ名を保持します。更新後にプロジェクトを開くと、新しいパスに基づく識別子が使用されます。古いプロジェクトのウィンドウは、都合の良いときに閉じてください。古い名前はディレクトリを確実に識別できないため、自動的な関連付けや移動は行われません。ディレクトリ名が重複している場合は、曖昧な名前ではなく、ピッカー、番号、または完全な正規パスを使用してください。

プロジェクトを開くと、設定されたプロジェクトリストと翻訳から`$XDG_RUNTIME_DIR/argvus/project-workspaces.json`が原子的に再生成されます。これはワークスペースのラベルやツールチップ用のセッションメタデータであり、設定ではありません。編集しないでください。リストや言語を変更した後にプロジェクトを再度開くと、ラベルが更新されます。`argvus`および`argvus-waybar`、さらにこの統合用に`argvus-taskbar`構成の両方を更新してください。
