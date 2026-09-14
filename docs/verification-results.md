# 検証結果（仕様書 §10）

検証日: 2026-09-14
環境: Windows 11 / Discord デスクトップ版 `app-1.0.9257`（起動中）/ Steam（起動中）
検証用コード: `verify/`（`dummy/` = ダミー exe、`harness.py` = ログ監視、`t_*.py` = 各テスト）

判定は `%APPDATA%\discord\logs\renderer_js.log` の `handleRunningGamesChange ... newPrimaryKey=<パス>:<ゲーム名>` と `[RunningGameHeartbeatManager] ... for game <ID>` で行った。
テストで作ったファイル・フォルダ・レジストリはすべて削除済み。

---

## まとめ

| # | 内容 | 結果 |
|---|---|---|
| V1 | `WS_EX_TOOLWINDOW`（タスクバー非表示）で検出されるか | **○** 約3秒で検出 |
| V2 | ウィンドウタイトルがゲーム名でも exe 名でも同じか | **○** どちらでも検出。タイトルは関係ない |
| V3 | 5本同時に動かして全部検出されるか | **○** 5本すべてにハートビートが出た |
| V4 | インストール済み STEAM ゲームのフォルダにハードリンクで置いて検出されるか | **○** 約2秒で検出 |
| V5 | `C:\Program Files (x86)\Steam\steamapps` 配下に管理者権限なしで書き込めるか | **○** `steamapps\` と `steamapps\common\` の両方に書き込めた |
| V6 | EXE 方式で `%LOCALAPPDATA%` 配下に置いて検出されるか | **○** 約1〜4秒で検出 |
| V7 | 停止してから「プレイ中」が消えるまでの時間 | **約5秒** |
| V8 | 未インストールの Steam ゲームに必要な登録情報 | **レジストリと appmanifest の両方が必要**（下記） |
| V9 | Steam 起動中に生成・削除したときの Steam の反応 | **一部確認**。30秒保持しても acf・レジストリの書き換えなし、Steam のログにも記録なし |
| V10 | 登録情報を作ってから検出されるまで | **Discord の再起動は不要**。登録直後にダミーを起動して約4秒で検出 |
| V11 | 所持・未インストールのゲームを起動中、Steam のライブラリ表示が変わるか | **変わらない**。「インストール」ボタンのまま。ダウンロードや更新も始まらない |
| V12 | 登録情報が残ったまま Steam を再起動したとき | **影響あり**。所持ゲームは「アップデート待機中」になり自動更新が予約される。未所持ゲームはライブラリに「購入」ボタン付きで出る。後始末しても Steam を再起動するまで表示は戻らない |
| V14 | Discord の検出後、ダミーを動かしたまま登録情報を消しても「プレイ中」が続くか | **続く**。削除後11分間ハートビートが出続けた。作り直しても何も起きない |

追加でわかったこと:
- **ウィンドウが表示状態（`WS_VISIBLE`）でないと検出されない**。`ShowWindow` を呼ばない非表示ウィンドウでは、20秒待っても検出されなかった
- 画面外 (-32000, -32000) の 1×1 ウィンドウでも、表示状態なら問題ない

---

## V1・V2・V6・V7: EXE 方式

配置先: `%LOCALAPPDATA%\DiscordIdlePicker\runtime\<DiscordID>\<登録パス>`（通常コピー）

| ゲーム | スタイル | タイトル | 検出 | 停止後に消えるまで |
|---|---|---|---|---|
| Factorio（`factorio.exe`） | app（`WS_EX_APPWINDOW`、`SW_SHOWMINNOACTIVE`） | exe 名 | ○ 4.0秒 | 5秒 |
| Celeste（`celeste.exe`） | tool（`WS_EX_TOOLWINDOW \| WS_EX_NOACTIVATE`、`WS_POPUP`、`SW_SHOWNOACTIVATE`） | exe 名 | ○ 3.0秒 | 5秒 |
| Hollow Knight（`hollow_knight.exe`） | hidden（ShowWindow なし） | exe 名 | **✗** 20秒待って未検出 | ― |
| Balatro（`balatro/balatro.exe`） | app | ゲーム名 | ○ 1.0秒 | 5秒 |

## V3: 同時起動

tool スタイルで Factorio → Celeste → Hollow Knight → Balatro → Hades の順に6秒間隔で起動。

- 起動のたびに `Running Games Changed` が1回ずつ出た
- メイン扱い（`visibleGame`）は途中で切り替わったり切り替わらなかったりする（Hollow Knight、Hades を起動してもメインは変わらなかった）
- 約5分半待ったところ、**5本すべての Discord ID でハートビートが1回ずつ出た**
- 逆順に止めると、メイン扱いが残っているゲームに順に移った

→ メイン扱いでないゲームは、ログの `newPrimaryKey` には出ない。個別に検出を確認するにはハートビート（5分ごと）を待つ必要がある。

## V4・V5: STEAM 方式

- V4: `D:\SteamLibrary\steamapps\common\WelcomeToTheGuildExplorers\discord-idle-picker-dummy.exe`（ハードリンク）で2.0秒で検出
- V5: 一時ファイルを作って消すテストで確認

## V8・V9・V10: 未インストールの Steam ゲーム

共通: `C:\Program Files (x86)\Steam\steamapps\common\DiscordIdlePicker_<appid>\discord-idle-picker-dummy.exe` を置き、登録情報を作った**直後**にダミーを起動。

生成したレジストリ（`HKCU\Software\Valve\Steam\Apps\<appid>`）:

```
Installed = 1 (DWORD)
Name      = <ゲーム名> (SZ)
```

生成した appmanifest（`C:\Program Files (x86)\Steam\steamapps\appmanifest_<appid>.acf`）:

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

| テスト | ゲーム（AppID） | レジストリ | appmanifest | 待ち時間 | 結果 |
|---|---|---|---|---|---|
| A | Pinball Spire（2601940） | ○ | ― | 90秒 | ✗ |
| B | Pinball Spire（2601940） | ― | ○ | 150秒 | ✗ |
| C | Pinball Spire（2601940） | ○ | ○ | ― | **○ 5.0秒** |
| A（再） | Sands（2230820） | ○ | ― | 60秒 | ✗ |
| B（再） | Those Who Remain（715380） | ― | ○ | 60秒 | ✗ |
| C（再） | Badlanders（1560500） | ○ | ○ | ― | **○ 3.5秒** |
| C（再） | Seasons after Fall（366320） | ○ | ○ | ― | **○ 4.0秒** |

V9 で確認できたこと（Badlanders で検出後30秒保持）:
- appmanifest の中身は Steam に書き換えられなかった
- レジストリの値も上書きされなかった
- `Steam\logs\*.txt` にその AppID を含む行は出なかった（生成中も削除後も）
- `steamapps\downloading\` に新しいフォルダはできなかった

V9 で**未確認**のこと:
- 長時間（数時間）置いたときに Steam がレジストリを書き換えるか

## V11: Steam のライブラリ表示

検証日: 2026-09-14（Steam 起動中）
ゲーム: Please, Don't Touch Anything（AppID 354240）。**所持していて未インストール**、exe パスなし（STEAM 方式）

未所持のゲームはもともとライブラリに出ないので、影響が出るとすれば所持しているゲーム。

手順:
1. `steam://nav/games/details/354240` でライブラリの詳細画面を開いてスクリーンショット
2. フォルダ・レジストリ・appmanifest を生成してダミーを起動（Discord は1.5秒で検出）
3. 生成から約10秒後と約35秒後に、詳細画面を開き直してスクリーンショット
4. 60秒間、5秒ごとに `steamapps\downloading\`、appmanifest の中身、`Steam\logs\*.txt` を確認
5. 後始末してから、もう一度スクリーンショット

結果:

| 時点 | ボタン | その他 |
|---|---|---|
| 生成前 | インストール（必要なディスク領域 140.46 MB） | ― |
| 生成・起動中（約10秒後、約35秒後） | **インストールのまま** | 「プレイ」や「停止」にもならない（サイドバーはこのゲームが画面外のため未確認） |
| 後始末後 | インストール | ― |

- `steamapps\downloading\` に新しいフォルダはできなかった（ダウンロード・更新は始まらない）
- appmanifest は Steam に書き換えられなかった
- Steam のログにその AppID が出たのは、こちらが `steam://nav` で画面を開いた記録だけだった

スクリーンショット: `verify/shots/owned_1_before_crop.png`、`owned_2_during_1_crop.png`、`owned_2_during_6_crop.png`、`owned_3_after_crop.png`

→ Steam 起動中は、Steam は生成した appmanifest を読み込んでいないと考えられる。Steam が appmanifest を読むのは起動時と思われるため、**appmanifest が残ったまま Steam を再起動した場合（V12）** に、ライブラリ表示の変化やダウンロードの開始が起きる可能性が残る。

## V12: 登録情報が残ったまま Steam を再起動

検証日: 2026-09-14
想定: アプリが強制終了して後始末できず、その状態で Steam が再起動された場合

対象:
- Please, Don't Touch Anything（354240）… **所持・未インストール**
- Badlanders（1560500）… **未所持**

手順:
1. 2本ぶんのフォルダ（ダミー exe 入り、起動はしない）・レジストリ・appmanifest を生成
2. `steam.exe -shutdown` で終了（4秒で終了）→ 起動
3. 起動から約2分間、10秒ごとに `steamapps\downloading\` を確認。その後 appmanifest・レジストリ・Steam のログを確認し、ライブラリ画面を撮影
4. Steam 起動中のまま後始末（フォルダ・レジストリ・appmanifest を削除）→ 15秒後に確認・撮影
5. Steam をもう一度終了 → 起動 → 約1分後に確認・撮影

### 結果

| 時点 | Please, Don't Touch Anything（所持） | Badlanders（未所持） |
|---|---|---|
| Steam 終了時 | appmanifest・レジストリとも変化なし | 同左 |
| Steam 起動後 | **「アップデート」ボタン、「アップデート待機中 0% 完了」** | **ライブラリの「カテゴリー未設定」に表示され、「購入」ボタン** |
| appmanifest | **Steam に書き換えられた**（下記） | 変化なし |
| レジストリ | `Updating=0`、`Running=0` が追加された | 同左 |
| ダウンロード | `downloading\` にフォルダはできなかった。ただし自動更新が予約された | なし |
| 起動中に後始末した後 | ファイルは消えたが、**表示は「アップデート待機中」のまま** | **ライブラリに残ったまま** |
| Steam 終了時（後始末後） | appmanifest の書き戻しはなかった | 同左 |
| もう一度再起動した後 | **「インストール」に戻った** | **ライブラリから消えた**（`steam://nav/games/details/1560500` で詳細画面が開かなくなった） |

Steam のログ（`content_log.txt`）:

```
[14:23:03] AppID 354240 state changed : Update Required,Fully Installed, (Update delayed for 572281 secs)
[14:23:03] AppID 354240 config changed : added depots 354241
[14:23:29] AppID 354240 state changed : Update Required,Fully Installed,Prefetching Info, (Update delayed for 572255 secs)
[14:23:31] AppID 354240 update prefetch finished : 134078768 bytes to download, 0 bytes to stage
```

Steam に書き換えられた appmanifest（抜粋）:

```
"StateFlags"		"6"            ← 4（インストール済み）+ 2（更新が必要）
"buildid"		"0"
"BytesToDownload"		"134078768"
"TargetBuildID"		"24615369"
"ScheduledAutoUpdate"		"1789935664"   ← 約6.6日後に自動更新を予約
```

スクリーンショット: `verify/shots/v12_*.png`

### わかったこと

- **Steam は appmanifest を起動時に読み込む**。起動中に作ったものは読まない（V11 と一致）
- **所持ゲーム**: Steam は「インストール済みだが更新が必要」と判断し、`DiscordIdlePicker_<appid>` フォルダへの自動更新を予約する。予約時刻になるか、ユーザーが「アップデート」を押すと、本物のゲーム（この例では約134MB）がそのフォルダにダウンロードされる可能性が高い
- **未所持ゲーム**: ライブラリに「購入」ボタン付きで表示される。ダウンロードは起きない
- **Steam は起動中、読み込んだ状態をメモリに持ち続ける**。ファイルとレジストリを消しても、Steam を再起動するまで表示は戻らない
- Steam の終了時に、消した appmanifest が書き戻されることはなかった
- 後始末 → Steam 再起動で完全に元に戻った

補足: Steam の再起動により、Steam Idle Picker で「実行中」にしていたゲームも止まった（検証前は Aseprite などが実行中だった）。

## V14: 検出後に登録情報を削除

検証日: 2026-09-14（Discord・Steam 起動中）
ゲーム: Turok（AppID 405820、Discord ID 1402416831914053733）。未所持・未インストール

手順と結果:

| 時刻 | 操作 | ログ |
|---|---|---|
| 14:30:59 | フォルダ・レジストリ・appmanifest を生成し、ダミーを起動 | ― |
| 14:31:04 | ― | 5.0秒で検出 |
| 14:31:05 | **レジストリと appmanifest を削除**（フォルダとダミーはそのまま） | ― |
| 14:36:03 | ― | ハートビート（1402416831914053733） |
| 14:41:03 | ― | ハートビート（1402416831914053733） |
| 14:31:05〜14:42:05 | 11分間監視 | `visible game became null` は0件。`Running Games Changed` も0件 |
| 14:42:05 | 登録情報を作り直す | 30秒間、ログに変化なし |
| 14:42:35 | 登録情報を再度削除し、ダミーを停止 | 14:42:36 に `visible game became null`（停止による） |

わかったこと:
- **Discord は検出した時点でゲームを確定し、その後は登録情報を見直さない**（少なくとも11分間）
- 登録情報を作り直しても、二重に検出されたりセッションが切り替わったりしない
- → 登録情報は検出の瞬間だけあればよい。検出後すぐ消せば、V12 のリスク（Steam 再起動時の影響）はほぼなくなる

検証しないことにしたもの（状況が限定的なため）:
- V13: 数時間単位でも「プレイ中」が続くか
- V15: Discord を再起動したとき、登録情報を作り直せば再検出されるか
