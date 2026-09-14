export interface Strings {
  RefreshList: string;
  SearchPlaceholder: string;
  ClearSearch: string;
  MaxSelection: string;
  IdleStart: string;
  IdleStop: string;
  LoadingList: string;
  LoadError: string;
  NoResults: string;
  FavoritesEmpty: string;
  FilterAll: string;
  FilterFavorites: string;
  FilterRunning: string;
  FilterExe: string;
  FilterSteam: string;
  AddFavorite: string;
  RemoveFavorite: string;
  SortName: string;
  SortMethod: string;
  SortId: string;
  StatusLabel: string;
  StatusRunning: string;
  StatusDetected: string;
  Detected: string;
  NotDetected: string;
  MenuCopyId: string;
  MenuOpenStore: string;
  MenuReveal: string;
  DiscordNotRunning: string;
  ListUnavailable: string;
  SteamNotFound: string;
  SteamRestartNeeded: string;
  ErrSteamNotFound: string;
  ErrFileExists: string;
  ErrAlreadyRegistered: string;
  ErrDummyMissing: string;
  ErrInvalidPath: string;
  ErrUnknownGame: string;
  ErrIo: string;
}

export const ja: Strings = {
  RefreshList: "一覧を再取得",
  SearchPlaceholder: "ゲームを検索...",
  ClearSearch: "検索をクリア",
  MaxSelection: "同時に起動できるのは 32 本までです",
  IdleStart: "▶ 起動",
  IdleStop: "■ 停止",
  LoadingList: "検出対象ゲームの一覧を取得中...",
  LoadError: "読み込みエラー: ",
  NoResults: "一致するゲームがありません",
  FavoritesEmpty: "☆ を押すとお気に入りに追加できます",
  FilterAll: "すべて",
  FilterFavorites: "お気に入り",
  FilterRunning: "起動中",
  FilterExe: "EXE",
  FilterSteam: "Steam",
  AddFavorite: "お気に入りに追加",
  RemoveFavorite: "お気に入りから外す",
  SortName: "名前",
  SortMethod: "方式",
  SortId: "AppID",
  StatusLabel: "ステータス: ",
  StatusRunning: " 稼働中",
  StatusDetected: "・Discord 検出 ",
  Detected: "Discord に検出されています",
  NotDetected: "Discord に検出されていません",
  MenuCopyId: "Discord ID をコピー",
  MenuOpenStore: "Steam ストアを開く",
  MenuReveal: "ダミーの場所を開く",
  DiscordNotRunning: "Discord が起動していません",
  ListUnavailable: "一覧を取得できません（⟳ で再試行）",
  SteamNotFound: "Steam が見つからないため、Steam のゲームは起動できません",
  SteamRestartNeeded: "前回の後始末をしました。Steam を再起動してください",
  ErrSteamNotFound: "Steam またはゲームのフォルダが見つかりません",
  ErrFileExists: "配置先に同じ名前のファイルがあります",
  ErrAlreadyRegistered: "Steam にインストール済みとして登録されています",
  ErrDummyMissing: "ダミーの実行ファイルが見つかりません",
  ErrInvalidPath: "実行ファイルのパスが不正です",
  ErrUnknownGame: "ゲームが一覧にありません",
  ErrIo: "起動に失敗しました: ",
};

export const en: Strings = {
  RefreshList: "Refresh list",
  SearchPlaceholder: "Search games...",
  ClearSearch: "Clear search",
  MaxSelection: "Up to 32 games can run at the same time",
  IdleStart: "▶ Start",
  IdleStop: "■ Stop",
  LoadingList: "Fetching detectable games...",
  LoadError: "Load error: ",
  NoResults: "No matching games",
  FavoritesEmpty: "Click ☆ to add favorites",
  FilterAll: "All",
  FilterFavorites: "Favorites",
  FilterRunning: "Running",
  FilterExe: "EXE",
  FilterSteam: "Steam",
  AddFavorite: "Add to favorites",
  RemoveFavorite: "Remove from favorites",
  SortName: "Name",
  SortMethod: "Method",
  SortId: "AppID",
  StatusLabel: "Status: ",
  StatusRunning: " running",
  StatusDetected: " · Detected by Discord: ",
  Detected: "Detected by Discord",
  NotDetected: "Not detected by Discord",
  MenuCopyId: "Copy Discord ID",
  MenuOpenStore: "Open Steam store page",
  MenuReveal: "Open dummy location",
  DiscordNotRunning: "Discord is not running",
  ListUnavailable: "Could not fetch the list (retry with ⟳)",
  SteamNotFound: "Steam was not found, so Steam games cannot be started",
  SteamRestartNeeded: "Cleaned up the previous session. Please restart Steam",
  ErrSteamNotFound: "Steam or the game folder was not found",
  ErrFileExists: "A file with the same name already exists",
  ErrAlreadyRegistered: "Steam already lists this game as installed",
  ErrDummyMissing: "Dummy executable not found",
  ErrInvalidPath: "Invalid executable path",
  ErrUnknownGame: "Game is not in the list",
  ErrIo: "Failed to start: ",
};

export function detectLanguage(): "ja" | "en" {
  return navigator.language.toLowerCase().startsWith("ja") ? "ja" : "en";
}

export function getStrings(lang: "ja" | "en"): Strings {
  return lang === "ja" ? ja : en;
}

/** Maps backend error codes (`code` or `code:detail`) to a message. */
export function errorText(t: Strings, error: unknown): string {
  const raw = String(error);
  const sep = raw.indexOf(":");
  const code = sep < 0 ? raw : raw.slice(0, sep);
  const detail = sep < 0 ? "" : raw.slice(sep + 1);
  switch (code) {
    case "steam_not_found":
      return t.ErrSteamNotFound;
    case "file_exists":
      return t.ErrFileExists;
    case "already_registered":
      return t.ErrAlreadyRegistered;
    case "dummy_missing":
      return t.ErrDummyMissing;
    case "invalid_path":
      return t.ErrInvalidPath;
    case "unknown_game":
      return t.ErrUnknownGame;
    case "io":
      return t.ErrIo + detail;
    default:
      return raw;
  }
}
