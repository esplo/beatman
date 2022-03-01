# beatman

beatmanは、BMSファイルの管理ツールです。

**※注意※**

**本ツールはファイルへの読み書きを多く行います。そのため、ファイルの消失やストレージの破損などが発生する可能性があります。
本ツールの使用によって生じた損害に対しいかなる責任も負いませんので、自己責任で利用してください。**

## Usage

### Windows

Powershellでexeファイルを実行。

```Powershell
> beatman.exe <OPTION> <SUBCOMMAND> <SUBCOMMAND_OPTION>
```

### Linux & Mac

WIP

## サブコマンド一覧

書き込みを伴うコマンドは、`dry-run` を付けることで書き込みを省き動作確認ができます。

以下、Windowsでの実行を例とします。

### check: 難易度表の中で持っていない譜面を検索

```Powershell
> beatman.exe --mydir O:\bms check --table-url https://stellabms.xyz/sl/table.html --level-limit 5
```

オプションの説明

- table-url
  - 難易度表のURL。 `table.html` または `score.json` を指定する。
  - Satellite (https://stellabms.xyz/sl/table.html)、Stella (https://stellabms.xyz/st/table.html) で動作確認済み
- level-limit
  - 検索したい上限難易度を指定。現状、数値のみ対応。

### install: zipファイルを展開して配置する

```Powershell
> beatman.exe --mydir O:\bms install --from C:\Users\puru\Downloads --recursive
```

オプションの説明

- from
  - インストールしたいzipファイルがあるディレクトリ
- recursive
  - 複数のディレクトリを対象にしてまとめてインストールする

インストールしたいzipは、事前に元譜面と差分のみを同じディレクトリに入れる必要があります。

`from`ディレクトリの構成（`--recursive`がない場合）

```text
<fromで指定したディレクトリ>/
--- bms.zip
--- diff1.zip
--- diff2.zip
--- other_files.bms
```

`from`ディレクトリの構成（`--recursive`がある場合）

```text
<fromで指定したディレクトリ>/
--- hoge/
--- --- bms.zip
--- --- diff1.zip
--- --- diff2.zip
--- --- other_files.bms
--- hoge2/
--- --- bms.zip
--- --- diff1.zip
```

なお、zip以外の圧縮ファイルには対応していないため、他は手動で展開する必要があります。

### organize: 重複フォルダのマージ、特定フォルダへの移動

```Powershell
> beatman.exe --mydir O:\bms organize --dest O:\custom
```

オプションの説明

- dest
  - 移動したい先のフォルダ。指定しない場合は、`mydir`と同じフォルダに展開する。

重複フォルダとみなすのは、同一のbmsファイルがあり、かつフォルダにあるファイル名の80%以上が一致している場合です。

なお、zipファイルの中身が入れ子のフォルダになっている構成などは上手く動かない可能性があります。

### beautify: フォルダのリネーム

```Powershell
> beatman.exe --mydir O:\bms rename
```

譜面情報を含まれている譜面から推定し、変更して綺麗にします。
`[アーティスト名] 譜面名` にリネームされます。同じフォルダ名になってしまう場合（.wav版と.ogg版がある状態など）、変更は行いません。

### task: (beatoraja限定) 目的に応じたカスタムフォルダを作成

```Powershell
> beatman.exe --mydir O:\bms task --table-url https://stellabms.xyz/sl/table.html --player-score-path "D:\beatoraja\player\player1\score.db" --songdata-path "D:\beatoraja\songdata.db" --folder-default-json "D:\beatoraja\table\default.json" --lower-limit-level 3 --target-lamp 4 --task-notes 50000
```

健康のために一日50,000ノーツ叩く場合など、日々のタスクを満たす譜面をまとめて1つのフォルダとして表示します。
beatorajaのデータベースとカスタムフォルダを利用しています。

既にbeatorajaで読み込まれている譜面のみ対象とされるため、楽曲追加時などは事前に "Update Database" を行ってください。

上記の例では、Satellite 3以上の難易度で未イージーの譜面から、ノーツ数合計が 50,000 を超えるまでピックアップし、カスタムフォルダを作成します。
通常、 "NEW FOLDER" 内に表示されます。

オプションの説明

- table-url
  - 難易度表のURL。 `table.html` または `score.json` を指定する。
  - Satellite (https://stellabms.xyz/sl/table.html)、Stella (https://stellabms.xyz/st/table.html) で動作確認済み
- player-score-path
  - `beatoraja/player/player1/score.db` などの位置にあるデータベースファイルのパス
- songdata-path
  - `beatoraja/songdata.db` などの位置にあるデータベースファイルのパス
- folder-default-json
  - `beatoraja/table/default.json` などの位置にある設定ファイルのパス
- lower-limit-level
  - 対象としたい下限難易度を指定。現状、数値のみ対応。
- target-lamp
  - "ASSIST_EASY", "EASY", "NORMAL", "HARD" のいずれかを指定
- task-notes
  - 目標の合計ノーツ数

## Troubleshoot

- (Windows) `Access is Denied` が出る

permissionが上手く設定されていないのかもしれません。原因は謎ですがろくにエラーメッセージが出ないので、とりあえずPowershellで該当のディレクトリに行って `icacls * /grant puru:f /T` などによりpermissionをいじる、`desktop.ini` など隠れているファイルを抹消する、などで対策が必要です。

開発者であれば、`RUST_LOG` 環境変数を `debug` にすると、どのファイルで引っかかっているかがわかります。
