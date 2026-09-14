# Discord Idle Picker 仕様書

版: 1.1
作成日: 2026-09-14
前提資料: [discord-activity-research.md](discord-activity-research.md)、[verification-results.md](verification-results.md)
UIベース: [Steam Idle Picker](https://github.com/4sou9/Steam_Idle_Picker) v2.1.1

### 改訂履歴

| 版 | 内容 |
|---|---|
| 0.1 | 初版 |
| 0.2 | 未インストールの Steam ゲームにも対応（§3.3、§6.3）。お気に入り機能を追加（§4.4、§4.6）。全件を一覧に表示する方針に変更 |
| 0.3 | exe パスも Steam AppID もないゲームは対象外にし、一覧に表示しない方針に変更 |
| 0.4 | 検索の表記ゆれ対応を削除（§5）。今後の拡張から自動停止タイマーを削除 |
| 0.5 | 検証結果を反映。未インストールの Steam ゲームはレジストリと appmanifest の両方を生成する方式に確定（§3.3）。ダミーのウィンドウスタイル確定（§6.1）。検出確認の判定を修正（§7） |
| 0.6 | V11 の検証結果を反映 |
| 0.7 | V12 の検証結果を反映。Steam 再起動への対策を追加（§3.4、§6.4） |
| 0.8 | V14 の検証結果を反映。登録情報は Discord の検出後すぐ削除する方式に変更し、Steam の監視を廃止。Discord 再起動時の再登録を追加（§3.4、§6.3、§6.4） |
| 0.9 | V13・V15 は状況が限定的なため検証しないことにした（§10） |
| 1.0 | M4 の実装に合わせて更新。既存のレジストリキーの扱い（§6.3）、台帳の形式と場所（§6.4、§8.2、§8.3）、アプリ起動時の Steam 再起動の案内（§4.5） |
| 1.1 | M5 の実装に合わせて更新。ファイル構成（§9）、開発の順番（§13） |

---

## 1. 概要

選んだゲームを Discord に「プレイ中」として検出させる Windows 用デスクトップアプリ。
ゲーム本体は起動せず、Discord の検出条件を満たすダミープロセスを動かす。

操作感は Steam Idle Picker と同じにする。「一覧から複数チェック → ▶ で一括起動 → ■ で一括停止」。

Discord の検出一覧のうち、exe パスか Steam AppID を持つゲームを、未所持・未インストールのものも含めてすべて一覧に表示し、起動できるようにする。

### 1.1 新規で作る理由（disactivity でカバーできないもの）

| 項目 | disactivity | 本アプリ |
|---|---|---|
| exe名で判定されるゲーム（10,450件） | ○ | ○ |
| exe名がなく Steam で判定されるゲーム・インストール済み | ✗ 一覧にも出ない | ○ |
| exe名がなく Steam で判定されるゲーム・未インストール（8,598件のほとんど） | ✗ 一覧にも出ない | ○ フォルダと登録情報を一時的に生成 |
| 同時起動 | 1本ずつ Run | 複数チェックして一括起動 |
| お気に入り | あり | あり（絞り込み表示つき） |
| Discord に検出されたかの確認 | なし | Discord のログから検出を確認して表示 |
| アプリが強制終了したとき | ダミーが残る | Job Object で道連れ終了、次回起動時に残骸を削除 |
| 日本語 | なし（英語／スペイン語） | 日本語／英語 |

### 1.2 対象外

- ユーザートークンを使う方式（セルフボット、クエストの自動完了 API 呼び出しなど）
- Rich Presence（`discord-ipc`）で任意の名前を出す機能
- 検出一覧にないソフト（Blender、OBS Studio など）
- exe パスも Steam AppID もないゲーム（約5,300件）。PlayStation や Epic Games Store など、別の方法で判定されているものと考えられ、プロセスを起動するだけでは検出させられない。一覧にも表示しない

---

## 2. 動作環境

| 項目 | 内容 |
|---|---|
| OS | Windows 10 / 11（x64） |
| 必須 | Discord デスクトップ版（起動していなくてもアプリは動くが、表示されない） |
| 必須（STEAM 方式のみ） | Steam のインストール。起動している必要はない |
| ネットワーク | 検出一覧の取得時のみ |

---

## 3. 判定方式と起動方法

調査メモ §3 の結論に基づき、各ゲームを次のどれかに分類する。

| 区分 | 条件 | 起動方法 |
|---|---|---|
| **EXE** | `executables` に `os == "win32"` の exe パスがある | ダミーを `<実行用フォルダ>\<DiscordID>\<登録パス>` に置いて起動（§6.2） |
| **STEAM**（インストール済み） | exe パスがなく、`third_party_skus` に Steam の AppID があり、その AppID がインストール済み | 本物のゲームフォルダにダミーを置いて起動（§6.3） |
| **STEAM**（未インストール） | 上と同じだが未インストール | フォルダと登録情報を一時的に生成し、そこにダミーを置いて起動（§6.3） |
| 対象外 | exe パスも Steam AppID もない | 一覧取得時に除外する（§1.2） |

- EXE と STEAM の両方の条件を満たすゲームは EXE を優先する（Steam 側に何も書き込まずに済むため）

### 3.1 exe パスの選び方（EXE）

disactivity の `select_best_executable` を踏襲し、次の順で1つ選ぶ。

1. `os == "win32"` かつ `name` が `>` で始まらない
2. `is_launcher == false` を優先
3. フォルダ階層（`/` の数）が少ないもの
4. 文字数が短いもの

### 3.2 インストール済み Steam ゲームの取得

1. レジストリ `HKCU\Software\Valve\Steam\SteamPath` から Steam のパスを得る
2. `<SteamPath>\steamapps\libraryfolders.vdf` を読んでライブラリ一覧を得る
3. 各ライブラリの `steamapps\appmanifest_*.acf` から `appid` と `installdir` を読む
4. `steamapps\common\<installdir>` が実在するものだけをインストール済みとする
5. 本アプリが生成した登録情報（§6.3 の台帳にあるもの）はインストール済みとして数えない

Steam Idle Picker の `services/vdf.rs` / `steam_library.rs` を流用する。

### 3.3 未インストールの Steam ゲームについて（重要）

未インストールのゲームは、フォルダを作るだけでは Discord に検出されない（調査メモ §3 テスト F）。
検証の結果、**レジストリと appmanifest の両方**があるときだけ検出された（[verification-results.md](verification-results.md) V8）。どちらか片方だけでは検出されない。

そのため、起動時に次の3つを生成し、停止時にすべて削除する。

| 生成するもの | 内容 |
|---|---|
| フォルダ | `<SteamPath>\steamapps\common\DiscordIdlePicker_<appid>\`（中にダミーを置く） |
| レジストリ | `HKCU\Software\Valve\Steam\Apps\<appid>` に `Installed=1`（DWORD）、`Name=<ゲーム名>`（SZ） |
| appmanifest | `<SteamPath>\steamapps\appmanifest_<appid>.acf`（下記の最小構成） |

```
"AppState"
{
	"appid"		"<appid>"
	"universe"		"1"
	"name"		"<ゲーム名>"
	"StateFlags"		"4"
	"installdir"		"DiscordIdlePicker_<appid>"
}
```

- `installdir` はデータに含まれないので、本アプリで `DiscordIdlePicker_<appid>` と決める（実在のゲームフォルダと名前がぶつからないようにするため）
- 生成先のライブラリは Steam 本体のライブラリ（`SteamPath`）に固定する
- Discord の再起動は不要。登録情報を生成した直後にダミーを起動して、約4秒で検出される（V10）
- Steam 起動中に生成・削除しても、acf やレジストリの書き換え、ダウンロードの開始は起きなかった（V9）。Steam のライブラリ表示も「インストール」のまま変わらない（V11）。ただし **登録情報が残ったまま Steam が再起動すると影響が出る**（V12、§3.4）

### 3.4 Steam 再起動のリスクと対策

V12 の検証で、登録情報が残ったまま Steam が起動すると次のことが起きるとわかった。

| ゲーム | Steam の反応 |
|---|---|
| 所持・未インストール | 「アップデート待機中」になり、自動更新が予約される。実行されると `DiscordIdlePicker_<appid>` に本物のゲームがダウンロードされる |
| 未所持 | ライブラリに「購入」ボタン付きで表示される |

どちらも、登録情報を消して **Steam をもう一度再起動すれば元に戻る**。Steam 起動中に消しただけでは表示は戻らない。

Steam は appmanifest を起動時にしか読まないので、「Steam が起動するときに登録情報が残っていない」ようにすれば防げる。

V14 の検証で、**Discord は検出した後に登録情報を消しても「プレイ中」を続ける**ことがわかった（削除後11分間、ハートビートが5分ごとに出続け、表示も消えなかった）。登録情報が必要なのは、Discord がダミーを検出する瞬間だけである。

そこで、次の方式にする。

1. **検出されたらすぐ登録情報を削除する**。登録情報が存在するのは、生成からダミーの検出まで（通常1〜5秒、最長15秒）だけにする（§6.3）
2. **Discord が再起動したら、登録情報を一時的に作り直す**。Discord は再起動後にもう一度検出し直すため。`Discord.exe` の起動を検知したら、起動中の未インストール STEAM ゲームについて「登録 → 検出を待つ → 削除」をもう一度行う
3. **アプリ起動時の後始末**（§6.4）で登録情報の残りを見つけて削除したとき、Steam が起動中なら、影響が出ている可能性がある。フッターに「Steam を再起動してください」と表示する

この方式なら、登録情報が残ったまま Steam が起動するのは、「登録から数秒のあいだにアプリが強制終了し、次にアプリを起動する前に Steam が再起動した」場合だけになる。Steam のプロセス監視は行わない。

---

## 4. 画面仕様

ウィンドウ構成は Steam Idle Picker と同一（枠なし・カスタムタイトルバー、Fluent 風、テーマと言語は Windows 設定に追従）。

### 4.1 レイアウト

```
┌────────────────────────────────────────────────────┐
│ [▶] [⟳]                               [─] [□] [×] │ ← タイトルバー（ドラッグで移動）
├────────────────────────────────────────────────────┤
│ [🔍 ゲームを検索...                  ] [すべて ▾]   │ ← ツールバー
├────────────────────────────────────────────────────┤
│         名前 ↑                        方式    AppID │ ← ソートヘッダー
│ [✓] ★ ● ✓ VRChat                       EXE   438100 │
│ [✓] ☆ ●   Slay the Spire II            EXE  2868840 │
│ [✓] ★ ●   Welcome to the Guild Expl…  STEAM 4327530 │
│ [ ] ★     Counter-Strike 2             EXE      730 │
│ [ ] ☆     Pinball Spire               STEAM 1234567 │
│  …                                                  │
├────────────────────────────────────────────────────┤
│ ステータス: 3/32 稼働中・Discord 検出 1     9/14 取得│ ← フッター
└────────────────────────────────────────────────────┘
```

既定サイズ 540×580、最小 480×400（方式列とお気に入り列のぶん Steam 版より 40px 広げる）。

### 4.2 タイトルバー

| 要素 | 動作 |
|---|---|
| ▶ / ■ ボタン | 稼働中が0件なら ▶（チェック済みを一括起動）。1件以上なら ■（全停止）。チェック0件かつ停止中は無効 |
| ⟳ ボタン | 検出一覧の再取得＋Steam ライブラリの再スキャン。実行中は回転アニメーション |
| ─ □ × | 最小化・最大化・閉じる。閉じると全停止してから終了 |

### 4.3 ツールバー

**検索ボックス**（§5 のルールで絞り込み）

**表示フィルタ**（Steam 版からの追加）

| 値 | 表示されるもの |
|---|---|
| すべて（既定） | 一覧の全件（EXE と STEAM） |
| お気に入り | お気に入りに登録したもの |
| 起動中 | ダミーが動いているもの |
| EXE | 方式が EXE のもの |
| Steam | 方式が STEAM のもの（インストール済み・未インストールの両方） |

検索とフィルタは同時に効く（例: 「お気に入り」の中から検索）。

### 4.4 ゲーム一覧

| 列 | 内容 |
|---|---|
| チェックボックス | 選択状態。上限32件。Steam が見つからないときは STEAM の行を無効にする |
| お気に入り | ★（登録済み）/ ☆（未登録）。クリックで切り替え。☆ は行にホバーしたときだけ表示し、★ は常に表示 |
| 状態 | 緑の ● = ダミー起動中。● の横の ✓ = Discord の検出を確認済み、黄色の ! = 未検出（§7） |
| 名前 | Discord 上の名前（`name`）。長い場合は末尾省略、ホバーで全文と別名を表示 |
| 方式 | `EXE` / `STEAM` の小さなバッジ |
| AppID | Steam AppID。ないものは空欄 |

**並び順**

1. チェック済み（Steam 版と同じく常に上に固定）
2. お気に入り
3. その他

それぞれのグループ内で、「名前」「AppID」のクリックで選んだ順に並べる（再クリックで逆順）。

**その他**

- 稼働中にチェックを入れるとそのゲームだけ即起動、外すとそのゲームだけ即停止（Steam 版と同じ）
- **仮想スクロール必須**（約19,000行。Steam 版のように全行を DOM に描画しない）

**右クリックメニュー**

| 項目 | 動作 |
|---|---|
| お気に入りに追加 / お気に入りから外す | ★ のクリックと同じ |
| Discord ID をコピー | `id` をクリップボードへ |
| Steam ストアを開く | `https://store.steampowered.com/app/<AppID>`（AppID があるときのみ） |
| ダミーの場所を開く | 起動中のみ。エクスプローラーで配置先を開く |

### 4.5 フッター

左: `ステータス: <稼働数>/32 稼働中・Discord 検出 <検出数>`
右: 以下の警告があるときだけ黄色で表示する（上ほど優先）。警告がなければ何も表示しない。

| 条件 | 表示 |
|---|---|
| Discord.exe が起動していない | Discord が起動していません |
| 一覧の取得に失敗し、キャッシュもない | 一覧を取得できません（⟳ で再試行） |
| Steam が見つからない | Steam が見つからないため、Steam のゲームは起動できません |
| アプリ起動時の後始末で登録情報を削除し、そのとき Steam が起動していた（§3.4） | 前回の後始末をしました。Steam を再起動してください（アプリを閉じるまで表示） |

Steam が見つからない場合、STEAM 方式の行はチェックできないようにする。

### 4.6 お気に入り

- 登録はチェック（起動対象の選択）とは独立。チェックを外してもお気に入りは残る
- 件数の上限なし
- 保存は `settings.json` の `favorites`（Discord ID の配列）
- 検出一覧から消えたゲームの ID は、⟳ で再取得したときに削除せず残す（一時的に一覧から外れた場合に備える）。一覧に存在しない ID は表示しないだけ

### 4.7 メッセージ（一覧が空のときのオーバーレイ）

| 状況 | 表示 |
|---|---|
| 初回起動・取得中 | 検出対象ゲームの一覧を取得中... |
| 取得失敗 | 読み込みエラー: <内容> |
| 検索結果0件 | 一致するゲームがありません |
| 「お気に入り」で0件 | ☆ を押すとお気に入りに追加できます |

---

## 5. 検索仕様

Steam 版と同じく、入力した文字列がゲーム名（`name` と `aliases`）に含まれるかで判定する。大文字・小文字は区別しない。
表記ゆれ（`2` と `II`、全角と半角、記号の有無など）は吸収しない。

加えて、クエリが数字だけの場合は Steam AppID と Discord ID の完全一致も対象にする。

---

## 6. ダミープロセス仕様

### 6.1 ダミー実行ファイル `dummy.exe`

Rust 製の小さな exe を1つだけ同梱し、ゲームごとに名前を変えてコピーして使う（Steam 版の `steam-idle.exe` に相当）。

| 項目 | 内容 |
|---|---|
| サブシステム | `windows`（コンソールなし） |
| 引数 | なし（タイトルは検出に関係しないため、exe 名をそのまま使う。V2） |
| ウィンドウ | 画面外 (-32000, -32000) に 1×1 のウィンドウを作成 |
| スタイル | `WS_EX_TOOLWINDOW \| WS_EX_NOACTIVATE`、`WS_POPUP`、`ShowWindow(SW_SHOWNOACTIVATE)`。タスクバーに出ない（V1 で検出を確認） |
| 表示状態 | **必ず表示状態にする**。`ShowWindow` を呼ばない非表示ウィンドウは検出されない |
| 処理 | メッセージループのみ。CPU 使用率ほぼ0 |
| 終了 | `WM_CLOSE` を受けたら終了 |

### 6.2 EXE 方式

| 項目 | 内容 |
|---|---|
| 配置先 | `%LOCALAPPDATA%\com.discordidlepicker.app\runtime\<DiscordID>\<登録パス>`（例: `...\398632010442211348\vrchat\vrchat.exe`） |
| コピー | 同じドライブならハードリンク、違えば通常コピー |
| 停止時 | `runtime\<DiscordID>\` ごと削除 |

`%TEMP%` は使わない（ディスククリーンアップで消える、ウイルス対策ソフトに疑われやすい）。

### 6.3 STEAM 方式

**インストール済みの場合**

| 項目 | 内容 |
|---|---|
| 配置先 | `<ライブラリ>\steamapps\common\<installdir>\discord-idle-picker-dummy.exe` |
| 制約 | **既存ファイルを絶対に上書きしない**。同名ファイルがあれば起動失敗 |
| 停止時 | ダミーの exe だけ削除。フォルダには触らない |

**未インストールの場合**（§3.3）

| 項目 | 内容 |
|---|---|
| 生成するフォルダ | `<SteamPath>\steamapps\common\DiscordIdlePicker_<appid>\` |
| 配置先 | 上のフォルダ内の `discord-idle-picker-dummy.exe` |
| 生成する登録情報 | レジストリと appmanifest の両方（§3.3） |
| 事前チェック | 次のどれかに当てはまれば起動失敗にする。生成先フォルダがすでにある（`file_exists`）／同じ AppID の `appmanifest_<appid>.acf` がどれかのライブラリにある（`already_registered`）／レジストリの `Apps\<appid>` が `Installed=1`（`already_registered`） |
| 既存のレジストリキー | プレイ歴のあるゲームは未インストールでも Steam が `Apps\<appid>` を持っていることがある（`Installed=0` や値なし）。この場合キーは作らず、`Installed` だけを 1 にし、削除時に元の値に戻す（値がなかった場合は値を削除）。`Name` は書き換えない |
| 削除の安全策 | appmanifest は中身に `DiscordIdlePicker_<appid>` を含む場合だけ削除する。既存キーの `Installed` は、現在 1 の場合だけ元に戻す（その間に本当にインストールされた記録を壊さないため） |
| 停止時 | 生成したフォルダごと削除し、登録情報が残っていれば削除する |

**生成と削除の順番**

起動: 台帳に記録 → フォルダ生成 → ダミー配置 → 登録情報を生成 → ダミー起動 → **検出を待つ → 登録情報を削除**
停止: ダミー終了 → 登録情報が残っていれば削除 → フォルダ削除 → 台帳から削除

「検出を待つ」の終わり方:

| 条件 | 動作 |
|---|---|
| ログにそのダミーのパス（`newPrimaryKey`）かゲームIDのハートビートが出た | すぐ削除 |
| ログに `Running Games Changed` が出た（メイン扱いでなくパスが出ない場合） | 2秒待って削除 |
| 15秒たっても何も出ない | 削除し、状態を「未検出」にする |
| Discord が起動していない | 登録情報は作らずにダミーだけ起動し、Discord の起動を待つ（§3.4 の2） |

台帳への記録を最初に行うことで、途中で落ちても次回起動時に後始末できるようにする。

### 6.4 起動・停止の共通処理

**起動**
1. 配置するファイル・フォルダ・登録情報を台帳 `placed.json` に記録する
2. §6.2 / §6.3 の手順で配置する
3. `CREATE_NO_WINDOW` で起動し、アプリ全体で1つの **Job Object**（`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`）に入れる
4. 失敗したら配置したものをすべて削除し、行の状態を戻して、フッターに理由を表示する

**停止**
1. ウィンドウに `WM_CLOSE` を送り、1秒待っても残っていれば強制終了
2. §6.2 / §6.3 の手順で削除する
3. 台帳から削除

**アプリ終了時**: 全停止してから終了（`CloseRequested` と `RunEvent::Exit` の両方で実行。Steam 版と同じ）

**強制終了されたとき**: Job Object によりダミーも自動で終了する。ファイルや登録情報は残るので、次回起動時に台帳を読んですべて削除する。台帳にないものには触らない。登録情報を削除したとき Steam が起動中なら、フッターに「Steam を再起動してください」と表示する（§3.4）。

**Discord の起動の監視**: `Discord.exe` のプロセスを2秒ごとに確認し、起動（または再起動）を検知したら、起動中の未インストール STEAM ゲームについて登録 → 検出を待つ → 削除を行う（§3.4）。複数ある場合はまとめて登録し、まとめて削除する。

### 6.5 状態の監視

1秒ごとにダミーの生存を確認し、終了していたら稼働中から外す（Steam 版の `get_idling_ids` ポーリングと同じ）。

---

## 7. Discord 検出の確認

`%APPDATA%\discord\logs
enderer_js.log` の末尾を監視し、ダミーが実際に検出されたかを判定する。

| ログ | 出るタイミング | 判定 |
|---|---|---|
| `[RunningGameStore] Running Games Changed` | 検出中のゲームが増減した直後（1〜5秒） | ゲームを特定できない。起動直後の目安にだけ使う |
| `handleRunningGamesChange ... newPrimaryKey=<フルパス小文字>:<ゲーム名>` | **メイン扱い**のゲームが変わったとき | パスが配置先と一致すれば検出済み |
| `[RunningGameHeartbeatManager] ... for game <ゲームID>` | 検出中の全ゲームについて**約5分ごと** | ゲームIDが一致すれば検出済み |

同時に複数動かすと、メイン扱いでないゲームは `newPrimaryKey` に出ない（V3）。そのため状態は次のように決める。

| 状態 | 条件 | 表示 |
|---|---|---|
| 検出済み | `newPrimaryKey` のパスかハートビートの ID が一致した | ✓ |
| 検出待ち | 起動後に `Running Games Changed` が出たが、まだ特定できていない | 表示なし |
| 未検出 | 起動後15秒以内に `Running Games Changed` が出なかった | 黄色の !（ホバーで「Discord に検出されていません」） |

- ファイルは起動時の末尾から読み始め、変更通知（`ReadDirectoryChangesW` か 2秒ポーリング）で追記分だけ読む
- ログのローテーションに備え、ファイルサイズが減ったら先頭から読み直す
- ログの書式は Discord の更新で変わりうるので、見つからない場合は検出表示を出さないだけにし、起動・停止には影響させない
- 停止後、Discord の「プレイ中」が消えるまで約5秒かかる（V7）

---

## 8. データ

### 8.1 検出一覧の取得

| 項目 | 内容 |
|---|---|
| URL | `GET https://discord.com/api/v9/applications/detectable`（認証不要） |
| タイミング | 起動時にキャッシュが24時間より古ければ裏で取得。⟳ で手動取得 |
| 失敗時 | キャッシュがあればそれを使い続ける |
| 絞り込み | exe パスも Steam AppID もないものは、この時点で除外する |
| 保存 | 必要な項目だけに絞って保存（元は約12.7MB） |

### 8.2 型

```ts
type Method = "exe" | "steam";

interface DetectableGame {
  id: string;            // Discord のアプリID（snowflake のため文字列）
  name: string;
  aliases: string[];
  exePath?: string;      // §3.1 で選んだ win32 の exe パス
  steamAppId?: number;
}

// Steam ライブラリと突き合わせた後、フロントに渡す形
interface GameEntry extends DetectableGame {
  method: Method;
  steamInstalled: boolean;   // STEAM 方式のとき、本物がインストール済みか
  steamInstallDir?: string;  // インストール済みの場合のフォルダ
}

interface GameCache {
  fetchedAt: string;         // ISO 8601
  games: DetectableGame[];
}

interface AppSettings {
  language: "ja" | "en";
  filter: "all" | "favorites" | "running" | "exe" | "steam";
  selectedGames: string[];   // Discord ID
  favorites: string[];       // Discord ID
}

interface RunningState {
  id: string;
  startedAt: string;
  detection: "pending" | "detected" | "notDetected";  // §7
}

// placed.json の1件分
interface PlacedEntry {
  id: string;
  files: string[];           // 削除するファイル（本物のゲームフォルダに置いたダミー）
  dirs: string[];            // 削除するフォルダ（生成したもののみ）
  registration: Registration | null;  // 登録情報が存在する間だけ入る
}

interface Registration {
  appId: number;
  manifest: string;          // 生成した appmanifest のパス
  keyCreated: boolean;       // Apps\<appid> キーを新しく作ったか
  prevInstalled: number | null;  // 既存キーの元の Installed（値がなければ null）
}
```

### 8.3 保存場所

`%APPDATA%\com.discordidlepicker.app\`

| ファイル | 内容 |
|---|---|
| `detectable_cache.json` | `GameCache` |
| `settings.json` | `AppSettings` |

`%LOCALAPPDATA%\com.discordidlepicker.app\placed.json` — 配置したものの台帳（`PlacedEntry[]`）。アプリ起動時に `runtime\` フォルダは丸ごと削除するため、台帳はその外に置く

`%LOCALAPPDATA%\com.discordidlepicker.app\runtime\` — ダミーの配置先（§6.2）

保存先はすべてバンドル ID 名のフォルダにする。アンインストーラーの「アプリのデータを削除」は `%APPDATA%\<バンドルID>` と `%LOCALAPPDATA%\<バンドルID>` だけを消すため。旧保存先 `%LOCALAPPDATA%\DiscordIdlePicker` が残っていれば、起動時にその台帳の後始末をしてからフォルダごと削除する

### 8.4 Tauri コマンド

Steam 版の `api.ts` を踏襲する。

| コマンド | 戻り値 | 内容 |
|---|---|---|
| `load_games` | `GameEntry[] \| null` | キャッシュ＋Steam スキャン結果 |
| `refresh_games` | `{ games, fetchedAt }` | 一覧の再取得＋Steam 再スキャン |
| `start_game(id)` | `Result<(), String>` | 失敗理由を返す（Steam 版は bool だった） |
| `stop_game(id)` | `()` | |
| `stop_all` | `()` | |
| `get_running` | `RunningState[]` | 1秒ごとに呼ぶ |
| `is_discord_running` | `bool` | |
| `load_settings` / `save_settings` | | お気に入りもここに含む |
| `copy_text` / `open_url` / `reveal_path` | | 右クリックメニュー用 |

---

## 9. 技術構成

| 層 | 技術 | Steam 版からの流用 |
|---|---|---|
| フロント | React 18 + TypeScript + Vite | `Titlebar.tsx`、`app.css`、`i18n/strings.ts` の仕組み、`App.tsx` の選択・ソート・ポーリングのロジック |
| 仮想スクロール | `@tanstack/react-virtual` | 新規 |
| バックエンド | Tauri 2（Rust） | `lib.rs` の構成、single-instance、終了時の全停止 |
| Steam 読み取り・書き込み | `winreg` + 自前 VDF パーサ／ライタ | `services/vdf.rs`、`services/steam_library.rs`（書き込みは新規） |
| HTTP | `reqwest`（rustls） | 新規 |
| Job Object | `windows` クレート | 新規 |
| ダミー | `dummy/`（別クレート、`windows` クレートのみ） | `steam-idle/` の置き換え。`scripts/copy-engine.mjs` を流用して `engine/` に配置 |
| 配布 | NSIS、ユーザー単位インストール | 同じ |

Steamworks（`steam_api64.dll`）は不要。

```
30_DISCORD_IDLE_PICKER/
├─ src/                       フロント
│  ├─ App.tsx                 状態管理・フィルタ・並び順・フッター
│  ├─ GameList.tsx            仮想スクロール一覧（§4.4）
│  ├─ ContextMenu.tsx         右クリックメニュー（§4.4）
│  ├─ Titlebar.tsx
│  ├─ api.ts / types.ts
│  ├─ i18n/strings.ts         日本語／英語、エラーコードの表示文言
│  └─ styles/app.css
├─ src-tauri/
│  ├─ src/
│  │  ├─ lib.rs / main.rs / commands.rs / models.rs
│  │  ├─ runner.rs            起動・停止、Job Object、Discord 起動の監視（§3.4、§6.4）
│  │  ├─ placement.rs         §6.2 / §6.3 の配置と削除、Steam 登録情報
│  │  ├─ ledger.rs            台帳 placed.json（§6.4）
│  │  ├─ discord_log.rs       ログ監視、検出状態（§7）
│  │  ├─ system.rs            プロセス確認、Job Object
│  │  └─ services/
│  │     ├─ detectable.rs     一覧の取得・exe 選択（§3.1、§8.1）
│  │     ├─ steam_library.rs  インストール済みゲームの取得（§3.2）
│  │     ├─ catalog.rs        一覧と Steam ライブラリの突き合わせ
│  │     └─ storage.rs        キャッシュ・設定・保存場所
│  └─ engine/dummy.exe        ビルド時に scripts/copy-engine.mjs が配置
├─ dummy/                     ダミー exe のクレート（§6.1）
├─ verify/                    検証用スクリプトとスクリーンショット
└─ docs/
```

---

## 10. 検証

V1〜V12、V14 は 2026-09-14 に検証済み。詳細は [verification-results.md](verification-results.md)。

| # | 内容 | 結果 | 反映先 |
|---|---|---|---|
| V1 | タスクバーに出ないウィンドウで検出されるか | ○ | §6.1 |
| V2 | ウィンドウタイトルは関係あるか | 関係ない | §6.1 |
| V3 | 5本同時に全部検出されるか | ○ | 上限32のまま |
| V4 | STEAM 方式でハードリンクでも検出されるか | ○ | §6.2 |
| V5 | Steam フォルダに管理者権限なしで書き込めるか | ○ | ― |
| V6 | `%LOCALAPPDATA%` 配下でも検出されるか | ○ | §6.2 |
| V7 | 停止後に表示が消えるまで | 約5秒 | §7 |
| V8 | 未インストールの Steam ゲームに必要な登録情報 | レジストリと appmanifest の両方 | §3.3 |
| V9 | Steam 起動中の Steam の反応 | 一部確認（変化なし） | §3.3 |
| V10 | Discord の再起動が必要か | 不要。約4秒で検出 | §3.3 |
| V11 | 起動中に Steam のライブラリ表示が変わるか（所持・未インストールのゲーム） | 変わらない。ダウンロードも始まらない | §3.3 |
| V12 | 登録情報が残ったまま Steam を再起動したとき | 影響あり。所持ゲームは自動更新が予約され、未所持ゲームはライブラリに出る。後始末＋Steam 再起動で戻る | §3.4 |
| V14 | 検出後に登録情報を消しても「プレイ中」が続くか | 続く（11分間ハートビート継続）。作り直しても変化なし | §3.4、§6.3 |

### 検証しないもの

状況が限定的なため、次の2つは検証しない。問題が出たら実装後に対応する。

| # | 内容 | 仕様上の扱い |
|---|---|---|
| V13 | 登録情報を消した状態で、数時間「プレイ中」が続くか（V14 は11分まで確認） | 続く前提で作る |
| V15 | Discord を再起動したとき、登録情報を作り直せば再検出されるか | §3.4 の2 のとおり作り直す（検証なし） |

---

## 11. 今後の拡張（v1 には入れない）

| 機能 | 内容 |
|---|---|
| Steam Idle Picker 連携 | 同じ AppID を Steamworks でも実行中にし、Steam のプレイ時間も同時に記録する |
| 他プラットフォーム対応 | PlayStation / Epic Games Store などで判定されるゲーム。プロセス起動とは別の方法が必要なので、調査から始める |
| 自動更新 | Tauri updater |

---

## 12. 注意事項

- ゲーム名の exe を作って起動するため、ウイルス対策ソフトに誤検出される可能性がある。配置先を固定し、可能ならコード署名する
- STEAM 方式では Steam のフォルダ（未インストールの場合は登録情報も）に一時的に書き込む。本アプリが作ったもの以外には触らず、停止時と次回起動時に必ず削除する
- Discord の検出方式やログ書式は予告なく変わる。§7 は壊れても本体機能に影響しない作りにする

---

## 13. 開発の順番

1. **M1（検証）**: 完了（V1〜V12、V14）
2. **M2**: 検出一覧の取得・キャッシュ、Steam 版 UI の移植、仮想スクロール、検索、EXE 方式の起動・停止（2026-09-14 完了）
3. **M3**: お気に入り、表示フィルタ、並び順（2026-09-14 完了）
4. **M4**: STEAM 方式（インストール済み・未インストール）、台帳、Job Object、残骸削除（2026-09-14 完了。Discord 未起動時の再登録は V15 と同じく未検証）
5. **M5**: Discord 検出の確認（§7）、右クリックメニュー、フッター警告、NSIS ビルド（2026-09-14 完了）
