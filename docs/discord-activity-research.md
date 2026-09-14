# Discord アクティビティ調査メモ

調査日: 2026-09-14
環境: Windows 11 / Discord デスクトップ版 `app-1.0.9257`

Steam Idle Picker の Discord 版を作る前段として、Discord の「プレイ中」表示、関連するバッジやタグ、既存ツールについて調べた結果をまとめる。

---

## 1. 「プレイ中」にする方法の比較

| 方式 | 表示されるもの | 備考 |
|---|---|---|
| **ゲーム検出（プロセス監視）** | 公式のゲーム名・アイコン。プレイ時間も記録される | 本調査の対象。ダミーのプロセスでも判定させられる |
| Rich Presence（ローカル通信 `discord-ipc-0`） | Developer Portal で作ったアプリの名前 | 好きなゲーム名は出せない。ゲームごとにアプリ登録が必要 |
| 登録済みのゲーム（手動） | 起動中の任意のプロセス。名前も変更できる | 公式の検出対象外なので、バッジには数えられない可能性が高い |
| ユーザートークンを使う方法（セルフボット） | 何でも可 |  |

---

## 2. 検出対象ゲームの一覧 API

| エンドポイント | 認証 | 内容 |
|---|---|---|
| `GET https://discord.com/api/v9/games/detectable` | 不要 | ゲームの一覧（約12.7MB） |
| `GET https://discord.com/api/v9/applications/detectable` | 不要 | 上と同じ内容（ID が完全に一致） |
| `GET https://discord.com/api/v9/applications/non-games/detectable` | 不要 | ゲーム以外（5件） |

### 件数（2026-09-14 時点）

| 区分 | 件数 |
|---|---|
| 全件 | **24,323** |
| Windows用のexe名が登録されている | **10,450** |
| exe名が登録されていない | **13,872** |
| └ そのうち Steam の AppID が紐づいている | 8,598 |

exe名のパターンが `>` で始まるものは0件だった。`os` の内訳は win32: 11,154、darwin: 67、linux: 8。

### 1件分のデータ例

```jsonc
// exe名で判定されるゲーム
{
  "id": "1479192099734945802",
  "name": "Slay the Spire II",          // 「2」ではなくローマ数字の「II」
  "aliases": [],
  "executables": [{ "is_launcher": false, "name": "slay the spire 2/slaythespire2.exe", "os": "win32" }],
  "third_party_skus": [{ "distributor": "steam", "id": "2868840" }]
}

// exe名がなく、Steam のフォルダで判定されるゲーム
{
  "id": "1473517429639348436",
  "name": "Welcome to the Guild Explorers",
  "executables": [],
  "third_party_skus": [{ "distributor": "steam", "id": "4327530" }]
}
```

### 注意点
- ゲーム名が一般的な表記と違うことがある（例: `Slay the Spire II`）。部分一致の検索だと「slay the spire 2」では見つからない。数字とローマ数字の揺れや、Steam AppID でも検索できるようにするべき
- Blender と OBS Studio は一覧に存在しない（検出対象外）

---

## 3. ゲーム検出の仕組み（実験で確認済み）

### 結論

| 方式 | 条件 | 対象 |
|---|---|---|
| **① exe名で判定** | 起動中のプロセスのパスが、登録された exe パスで**終わっている**。置き場所は問わない | exe名があるゲーム（10,450件） |
| **② Steam のフォルダで判定** | **インストール済み**の Steam ゲームのフォルダ `<ライブラリ>/steamapps/common/<installdir>/` の中で起動している。**exe名は何でもいい** | exe名がなく Steam AppID があるゲーム |

**どちらの方式でも、プロセスがウィンドウを持っていることが必須。** ウィンドウのないプロセスは判定対象にならない。

### 実験の内容
ゲーム: Welcome to the Guild Explorers（Steam 4327530、方式②のゲーム）
本物のexe: `D:\SteamLibrary\steamapps\common\WelcomeToTheGuildExplorers\RPGDetecive.exe`（Godot製で、メタ情報も署名もない）

| テスト | ウィンドウ | 置き場所 | exe名 | Steam上で実行中 | 結果 |
|---|---|---|---|---|---|
| A | なし | ゲームフォルダ内 | `probe.exe` | なし | ✗ |
| B | なし | Steamの外 | `RPGDetecive.exe` | なし | ✗ |
| C | なし | ゲームフォルダ内（本物と入れ替え） | `RPGDetecive.exe` | なし | ✗ |
| D | ― | ダミーなし（Steam Idle Picker で起動しただけ） | ― | あり | ✗ |
| E | なし | ゲームフォルダ内 | `probe.exe` | あり | ✗ |
| 確認 | **あり** | Steamの外 | `factorio/factorio.exe` | ― | **✓ 約1秒で検出** |
| **A2** | **あり** | **ゲームフォルダ内** | `probe.exe` | なし | **✓ 検出** |
| B2 | あり | Steamの外 | `WelcomeToTheGuildExplorers/RPGDetecive.exe` | なし | ✗ |
| F | あり | 未インストールのゲーム（Pinball Spire）のフォルダを作っただけ | `probe.exe` | なし | ✗ |

わかったこと:
- A〜E が失敗したのは、ウィンドウがなかったから（判定対象外）
- 方式②では、Steam上で実行中かどうか（レジストリの `RunningAppID` や `Apps\<id>\Running`）は**関係ない**
- 方式②では、Steam の管理ファイル `appmanifest_<appid>.acf` やレジストリにインストール済みの記録があることが**必須**。フォルダを作るだけでは判定されない
- Steam Idle Picker（Steamworks API で「実行中」にする方式）だけでは、Discord には表示されない

### Discord の内部モジュールにあった手がかり
`%LOCALAPPDATA%\Discord\app-<ver>\modules\discord_utils-1\discord_utils\discord_utils.node` に次の文字列が含まれていた。

- `SteamObserver::DetectSteamGame(RunningProcess)`、`TryDetectingSteamGame`、`SteamObserver::updateInstalledSkus`
- `SOFTWARE\Valve\Steam`、`Software\Valve\Steam\Apps\%d`、`InstallDir`、`RunningAppID`、`SteamPath`
- `\config\libraryfolders.vdf`、`/steamapps/common/`
- `GetWindowTextW`、`getExecutableFingerprintForProcess`、`executableFingerprint`

`discord_game_utils.node` には `ScanSteamGames`、`AppExePath`、`steam://run/` があった。

### 検出を確認する方法（ログ）
`%APPDATA%\discord\logs\renderer_js.log`

```
[Clips] decider: handleRunningGamesChange visibleGame=<ゲーム名> newPrimaryKey=<exeのフルパス(小文字)>:<ゲーム名> ...
[RunningGameStore] Running Games Changed [object Object]
[RunningGameHeartbeatManager] Failed to get performance snapshot for game <ゲームID> ...   ← 検出中は5分ごとに出る
```

- 検出されると、`Running Games Changed` と `Clips decider` がほぼ即座に出る
- ほかのゲームがメイン扱いのときは、`visibleGame` が切り替わらないことがある。確実に判定するならハートビートの行を見る

---

## 4. disactivity のダミー方式（holasoyender/disactivity）

リポジトリ: https://github.com/holasoyender/disactivity （Tauri + TypeScript、MIT）

1. `games/detectable` と `applications/non-games/detectable` を取得する。**exe名がないゲームは除外**するので、方式②のゲームはリストに出ない。一覧は2日間キャッシュする
2. `select_best_executable`: `os == "win32"` で `>` で始まらないものから、フォルダの階層がいちばん浅く、名前がいちばん短いものを選ぶ
3. `%TEMP%\disactivity\<ゲームID>\<登録パス>` にフォルダを作る
   - 例: `C:\Users\<user>\AppData\Local\Temp\disactivity\398632010442211348\vrchat\vrchat.exe`
4. アプリに埋め込んだ `slave.exe` をその名前で書き出して起動する
5. slave は画面外（-32000, -32000）に1×1ピクセルのウィンドウを作る
   - スタイル: `WS_EX_APPWINDOW | WS_EX_NOACTIVATE`、`WS_POPUP | WS_OVERLAPPEDWINDOW`、`SW_SHOWMINNOACTIVE`
   - ウィンドウタイトルはexe名
6. 停止時やアプリ終了時に、プロセスを止めて `%TEMP%\disactivity\<ゲームID>` を削除する
7. 検索はゲーム名・ID・別名に、入力した文字がそのまま含まれるかだけで判定する

置き場所が一時フォルダでも方式①は通る。ただし方式②には原理的に対応できない。

---

## 5. 既存の類似ツール（GitHub、2026-09 時点）

| リポジトリ | ★ | 技術 | 特徴 |
|---|---|---|---|
| holasoyender/disactivity | 29 | Tauri + TS | GUI。一覧から選んで Run で起動する |
| markterence/discord-quest-completer | 867 | Tauri + Vue | GUI。クエスト用途が中心 |
| Jeardey/discord-fake-game-launcher | 88 | Electron + C# | `%APPDATA%` にダミーを作る |
| strykey/orbshacker | 204 | Python（CLI） | 複数同時起動に対応。偽の appmanifest を作る「Steam Quest Mode」がある |
| Masterain98/discord-quest-helper | 100 | Tauri + Vue | トークンでログインする方式（規約リスクが高い） |
| devAxri/Discord-Game-Spoofer ほか | 少数 | C# / Python | 小規模なもの |

---

## 6. プロフィールのアクティビティ表示

### 6.1 ゲームごとに付くタグ（最近のアクティビティ／メンバーリスト）
対象は直近30日間。公式の全種類の一覧は見つからなかった。

| 表示 | 意味 | 確認できたか |
|---|---|---|
| 〇時間マラソン | そのゲームを何時間続けてプレイしたか | 記事で確認済み |
| 〇日連続 | そのゲームを何日続けてプレイしたか | 記事で確認済み |
| 〇か月ぶり | しばらく間が空いてから再開した | 記事で確認済み |
| かけだしプレイヤー | そのゲームを始めたばかり | 未確認（公式の説明なし） |
| よく遊ぶゲーム | いちばん多く遊んでいるゲーム | 未確認 |

---

