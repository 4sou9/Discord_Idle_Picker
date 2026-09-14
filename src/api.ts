import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, GamesPayload, RunningState } from "./types";

export const api = {
  loadGames: () => invoke<GamesPayload | null>("load_games"),
  refreshGames: () => invoke<GamesPayload>("refresh_games"),
  startGame: (id: string) => invoke<void>("start_game", { id }),
  stopGame: (id: string) => invoke<void>("stop_game", { id }),
  stopAll: () => invoke<void>("stop_all"),
  getRunning: () => invoke<RunningState[]>("get_running"),
  isDiscordRunning: () => invoke<boolean>("is_discord_running"),
  steamRestartNeeded: () => invoke<boolean>("steam_restart_needed"),
  openSteamStore: (appId: number) => invoke<void>("open_steam_store", { appId }),
  revealDummy: (id: string) => invoke<void>("reveal_dummy", { id }),
  loadSettings: () => invoke<AppSettings>("load_settings"),
  saveSettings: (settings: AppSettings) => invoke<void>("save_settings", { settings }),
};
