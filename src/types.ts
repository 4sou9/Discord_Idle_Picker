export type Method = "exe" | "steam";

export interface GameEntry {
  id: string;
  name: string;
  aliases: string[];
  exePath?: string;
  steamAppId?: number;
  method: Method;
  steamInstalled: boolean;
  steamInstallDir?: string;
}

export interface GamesPayload {
  games: GameEntry[];
  fetchedAt: string;
  steamFound: boolean;
}

export type FilterMode = "all" | "favorites" | "running" | "exe" | "steam";

export interface AppSettings {
  language: string;
  filter: FilterMode;
  selectedGames: string[];
  favorites: string[];
}

export type Detection = "pending" | "detected" | "notDetected";

export interface RunningState {
  id: string;
  startedAt: string;
  detection: Detection;
}

export type SortMode = "none" | "name" | "id";
