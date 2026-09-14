import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import ContextMenu, { type MenuItem } from "./ContextMenu";
import GameList from "./GameList";
import { detectLanguage, errorText, getStrings } from "./i18n/strings";
import type { AppSettings, Detection, FilterMode, GameEntry, RunningState, SortMode } from "./types";
import Titlebar from "./Titlebar";
import "./styles/app.css";

const MAX_SELECTION = 32;
const CACHE_TTL_MS = 24 * 60 * 60 * 1000;
const NOTICE_MS = 6000;
const lang = detectLanguage();
const t = getStrings(lang);

const FILTERS: { value: FilterMode; label: string }[] = [
  { value: "all", label: t.FilterAll },
  { value: "favorites", label: t.FilterFavorites },
  { value: "running", label: t.FilterRunning },
  { value: "exe", label: t.FilterExe },
  { value: "steam", label: t.FilterSteam },
];

/** Stable fingerprint of the running list, to skip no-op state updates while polling. */
function runningKey(states: RunningState[]) {
  return states
    .map((s) => `${s.id}:${s.detection}`)
    .sort()
    .join(",");
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const area = document.createElement("textarea");
    area.value = text;
    document.body.appendChild(area);
    area.select();
    document.execCommand("copy");
    area.remove();
  }
}

const ICON_PLAY = "\uE768";
const ICON_STOP = "\uE71A";
const ICON_REFRESH = "\uE72C";
const ICON_SEARCH = "\uE721";
const ICON_CLEAR = "\uE711";

function sortIcon(active: boolean, ascending: boolean) {
  if (!active) return "";
  return ascending ? " ↑" : " ↓";
}

export default function App() {
  const [games, setGames] = useState<GameEntry[]>([]);
  const [steamFound, setSteamFound] = useState(true);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [favoriteIds, setFavoriteIds] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<FilterMode>("all");
  const [runningIds, setRunningIds] = useState<Set<string>>(new Set());
  const [detections, setDetections] = useState<Map<string, Detection>>(new Map());
  const [menu, setMenu] = useState<{ game: GameEntry; x: number; y: number } | null>(null);
  const [searchText, setSearchText] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("none");
  const [sortAscending, setSortAscending] = useState(true);
  const [loadMessage, setLoadMessage] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [discordRunning, setDiscordRunning] = useState(true);
  const [notice, setNotice] = useState("");
  const [steamRestartNeeded, setSteamRestartNeeded] = useState(false);

  const settingsRef = useRef<AppSettings>({ language: lang, filter: "all", selectedGames: [], favorites: [] });
  const gamesById = useMemo(() => new Map(games.map((g) => [g.id, g])), [games]);

  const isRunning = runningIds.size > 0;
  const runningKeyRef = useRef("");
  const searchInputRef = useRef<HTMLInputElement>(null);

  const applyRunning = useCallback((states: RunningState[]) => {
    const key = runningKey(states);
    if (key === runningKeyRef.current) return;
    runningKeyRef.current = key;
    setRunningIds(new Set(states.map((s) => s.id)));
    setDetections(new Map(states.map((s) => [s.id, s.detection])));
  }, []);

  // The webview's own menu (Reload, Inspect, ...) is not useful here; inputs keep theirs.
  useEffect(() => {
    const onContextMenu = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest("input, textarea")) return;
      e.preventDefault();
    };
    window.addEventListener("contextmenu", onContextMenu);
    return () => window.removeEventListener("contextmenu", onContextMenu);
  }, []);

  const showNotice = useCallback((message: string) => {
    setNotice(message);
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(""), NOTICE_MS);
    return () => window.clearTimeout(timer);
  }, [notice]);

  // ── Refresh ──────────────────────────────────────────────────────────
  const refreshGames = useCallback(async (hasCache: boolean) => {
    setIsRefreshing(true);
    if (!hasCache) setLoadMessage(t.LoadingList);
    try {
      const payload = await api.refreshGames();
      setGames(payload.games);
      setSteamFound(payload.steamFound);
      setLoadMessage("");
    } catch (e) {
      // Keep showing the cache if there is one (spec §8.1).
      if (hasCache) showNotice(t.LoadError + String(e));
      else setLoadMessage(t.LoadError + String(e));
    } finally {
      setIsRefreshing(false);
    }
  }, [showNotice]);

  // ── Initial load ──────────────────────────────────────────────────────
  useEffect(() => {
    (async () => {
      const [payload, settings] = await Promise.all([api.loadGames(), api.loadSettings()]);
      settingsRef.current = settings;
      setSelectedIds(new Set(settings.selectedGames));
      setFavoriteIds(new Set(settings.favorites));
      if (FILTERS.some((f) => f.value === settings.filter)) setFilter(settings.filter);
      if (payload) {
        setGames(payload.games);
        setSteamFound(payload.steamFound);
      }
      setSteamRestartNeeded(await api.steamRestartNeeded());
      applyRunning(await api.getRunning());

      const stale = !payload || Date.now() - new Date(payload.fetchedAt).getTime() > CACHE_TTL_MS;
      if (stale) void refreshGames(payload !== null);
    })();
  }, [refreshGames, applyRunning]);

  // ── Polling while running (replace runningIds wholesale) ─────────────
  useEffect(() => {
    if (!isRunning) return;
    const timer = window.setInterval(async () => {
      applyRunning(await api.getRunning());
    }, 1000);
    return () => window.clearInterval(timer);
  }, [isRunning, applyRunning]);

  useEffect(() => {
    const check = () => void api.isDiscordRunning().then(setDiscordRunning);
    check();
    const timer = window.setInterval(check, 5000);
    return () => window.clearInterval(timer);
  }, []);

  const updateSettings = useCallback((patch: Partial<AppSettings>) => {
    const settings: AppSettings = { ...settingsRef.current, ...patch };
    settingsRef.current = settings;
    void api.saveSettings(settings);
  }, []);

  const changeFilter = useCallback(
    (value: FilterMode) => {
      setFilter(value);
      updateSettings({ filter: value });
    },
    [updateSettings]
  );

  // Favorites are independent of selection. IDs that vanish from the list are kept
  // (spec §4.6), so this only ever adds or removes the clicked ID.
  const toggleFavorite = useCallback(
    (id: string) => {
      setFavoriteIds((prev) => {
        const next = new Set(prev);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        updateSettings({ favorites: Array.from(next) });
        return next;
      });
    },
    [updateSettings]
  );

  const startOne = useCallback(
    async (id: string) => {
      try {
        await api.startGame(id);
        return true;
      } catch (e) {
        const name = gamesById.get(id)?.name ?? id;
        showNotice(`${name}: ${errorText(t, e)}`);
        return false;
      }
    },
    [gamesById, showNotice]
  );

  // ── Selection ────────────────────────────────────────────────────────
  const toggleSelected = useCallback(
    (id: string, checked: boolean) => {
      if (checked && !selectedIds.has(id) && selectedIds.size >= MAX_SELECTION) {
        showNotice(t.MaxSelection);
        return;
      }
      setSelectedIds((prev) => {
        if (checked && prev.size >= MAX_SELECTION) return prev; // hard cap, ignore
        const next = new Set(prev);
        if (checked) next.add(id);
        else next.delete(id);
        updateSettings({ selectedGames: Array.from(next) });
        return next;
      });

      if (isRunning) {
        if (checked) {
          void startOne(id).then(async () => applyRunning(await api.getRunning()));
        } else {
          void api.stopGame(id).then(async () => applyRunning(await api.getRunning()));
        }
      }
    },
    [isRunning, updateSettings, startOne, applyRunning, selectedIds, showNotice]
  );

  // ── Start / stop ─────────────────────────────────────────────────────
  const startAll = useCallback(async () => {
    for (const id of selectedIds) {
      if (!gamesById.has(id)) continue;
      await startOne(id);
    }
    applyRunning(await api.getRunning());
  }, [selectedIds, gamesById, startOne, applyRunning]);

  const stopAll = useCallback(async () => {
    await api.stopAll();
    applyRunning(await api.getRunning());
  }, [applyRunning]);

  const toggleRun = useCallback(() => {
    if (isRunning) void stopAll();
    else void startAll();
  }, [isRunning, startAll, stopAll]);

  // ── Sort ─────────────────────────────────────────────────────────────
  const toggleSort = useCallback(
    (mode: SortMode) => {
      if (sortMode === mode) {
        setSortAscending((a) => !a);
      } else {
        setSortMode(mode);
        setSortAscending(true);
      }
    },
    [sortMode]
  );

  const searchKeys = useMemo(
    () => games.map((g) => [g.name, ...g.aliases].map((s) => s.toLowerCase())),
    [games]
  );

  // ── Derived display list — pure, never touches selection/run state ────
  const visibleGames = useMemo(() => {
    const query = searchText.trim().toLowerCase();
    let result = games;
    if (query) {
      const numeric = /^\d+$/.test(query);
      result = games.filter(
        (g, i) =>
          searchKeys[i].some((key) => key.includes(query)) ||
          (numeric && (g.id === query || String(g.steamAppId) === query))
      );
    }

    switch (filter) {
      case "favorites":
        result = result.filter((g) => favoriteIds.has(g.id));
        break;
      case "running":
        result = result.filter((g) => runningIds.has(g.id));
        break;
      case "exe":
      case "steam":
        result = result.filter((g) => g.method === filter);
        break;
      default:
        break;
    }

    let sorted = result;
    if (sortMode === "name") {
      const collator = new Intl.Collator(undefined, { sensitivity: "base" });
      sorted = [...result].sort((a, b) => collator.compare(a.name, b.name));
    } else if (sortMode === "id") {
      sorted = [...result].sort((a, b) => (a.steamAppId ?? Infinity) - (b.steamAppId ?? Infinity));
    }
    if (!sortAscending && sortMode !== "none") sorted = [...sorted].reverse();

    // Spec §4.4: checked → favorites → others, each keeping the chosen sort.
    if (selectedIds.size === 0 && favoriteIds.size === 0) return sorted;
    const checked: GameEntry[] = [];
    const favorites: GameEntry[] = [];
    const others: GameEntry[] = [];
    for (const g of sorted) {
      if (selectedIds.has(g.id)) checked.push(g);
      else if (favoriteIds.has(g.id)) favorites.push(g);
      else others.push(g);
    }
    return checked.concat(favorites, others);
  }, [games, searchKeys, searchText, filter, sortMode, sortAscending, selectedIds, favoriteIds, runningIds]);

  const emptyMessage =
    filter === "favorites" && !games.some((g) => favoriteIds.has(g.id)) ? t.FavoritesEmpty : t.NoResults;
  const overlay = loadMessage || (games.length > 0 && visibleGames.length === 0 ? emptyMessage : "");

  // Right side of the footer: only notices and warnings, most important first.
  const footerWarning = (() => {
    if (notice) return notice;
    if (steamRestartNeeded) return t.SteamRestartNeeded;
    if (!discordRunning) return t.DiscordNotRunning;
    if (games.length === 0 && !isRefreshing && loadMessage.startsWith(t.LoadError)) return t.ListUnavailable;
    if (games.length > 0 && !steamFound) return t.SteamNotFound;
    return "";
  })();

  const detectedCount = Array.from(detections.values()).filter((d) => d === "detected").length;

  const openMenu = useCallback((game: GameEntry, x: number, y: number) => setMenu({ game, x, y }), []);
  const closeMenu = useCallback(() => setMenu(null), []);

  const menuItems = (game: GameEntry): MenuItem[] => {
    const items: MenuItem[] = [
      {
        label: favoriteIds.has(game.id) ? t.RemoveFavorite : t.AddFavorite,
        onSelect: () => toggleFavorite(game.id),
      },
      { label: t.MenuCopyId, onSelect: () => void copyText(game.id) },
    ];
    if (game.steamAppId !== undefined) {
      const appId = game.steamAppId;
      items.push({
        label: t.MenuOpenStore,
        onSelect: () => void api.openSteamStore(appId).catch((e) => showNotice(errorText(t, e))),
      });
    }
    if (runningIds.has(game.id)) {
      items.push({
        label: t.MenuReveal,
        onSelect: () => void api.revealDummy(game.id).catch((e) => showNotice(errorText(t, e))),
      });
    }
    return items;
  };

  const startableSelected = Array.from(selectedIds).some((id) => gamesById.has(id));

  return (
    <div className="app">
      <Titlebar>
        <button
          className={"icon-button primary" + (isRunning ? " stop" : "")}
          onClick={toggleRun}
          disabled={!isRunning && !startableSelected}
          title={isRunning ? t.IdleStop : t.IdleStart}
        >
          {isRunning ? ICON_STOP : ICON_PLAY}
        </button>
        <button
          className={"icon-button small" + (isRefreshing ? " spinning" : "")}
          onClick={() => void refreshGames(games.length > 0)}
          disabled={isRefreshing}
          title={t.RefreshList}
        >
          {ICON_REFRESH}
        </button>
      </Titlebar>

      <div className="toolbar">
        <div className="search-box">
          <span className="icon">{ICON_SEARCH}</span>
          <input
            ref={searchInputRef}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder={t.SearchPlaceholder}
          />
          {searchText && (
            <button
              className="search-clear"
              onClick={() => {
                setSearchText("");
                searchInputRef.current?.focus();
              }}
              title={t.ClearSearch}
              aria-label={t.ClearSearch}
            >
              {ICON_CLEAR}
            </button>
          )}
        </div>
        <select
          className="filter-select"
          value={filter}
          onChange={(e) => changeFilter(e.target.value as FilterMode)}
        >
          {FILTERS.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
      </div>

      <div className="list-panel">
        <div className="sort-header">
          <button className="sort-name" onClick={() => toggleSort("name")}>
            {t.SortName}
            {sortIcon(sortMode === "name", sortAscending)}
          </button>
          <span className="sort-method">{t.SortMethod}</span>
          <button className="sort-id" onClick={() => toggleSort("id")}>
            {t.SortId}
            {sortIcon(sortMode === "id", sortAscending)}
          </button>
        </div>

        <GameList
          games={visibleGames}
          selectedIds={selectedIds}
          runningIds={runningIds}
          detections={detections}
          favoriteIds={favoriteIds}
          steamFound={steamFound}
          overlay={overlay}
          strings={t}
          onToggle={toggleSelected}
          onToggleFavorite={toggleFavorite}
          onContextMenu={openMenu}
        />
      </div>

      <div className="footer">
        <span className="footer-status">
          {t.StatusLabel}
          <strong>
            {runningIds.size}/{MAX_SELECTION}
          </strong>
          {t.StatusRunning}
          {t.StatusDetected}
          <strong>{detectedCount}</strong>
        </span>
        <span className="footer-info warn" title={footerWarning}>
          {footerWarning}
        </span>
      </div>

      {menu && <ContextMenu x={menu.x} y={menu.y} items={menuItems(menu.game)} onClose={closeMenu} />}
    </div>
  );
}
