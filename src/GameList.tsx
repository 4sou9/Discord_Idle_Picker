import { memo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { Strings } from "./i18n/strings";
import type { Detection, GameEntry } from "./types";

const ROW_HEIGHT = 32;

const ICON_CHECKED = "\uE73A";
const ICON_UNCHECKED = "\uE739";
const ICON_RUNNING = "\uEA3B";
const ICON_STAR = "\uE734";
const ICON_DETECTED = "\uE73E";
const ICON_NOT_DETECTED = "\uE7BA";
const ICON_STAR_FILLED = "\uE735";

export default function GameList({
  games,
  selectedIds,
  runningIds,
  detections,
  favoriteIds,
  steamFound,
  overlay,
  strings,
  onToggle,
  onToggleFavorite,
  onContextMenu,
}: {
  games: GameEntry[];
  selectedIds: Set<string>;
  runningIds: Set<string>;
  detections: Map<string, Detection>;
  favoriteIds: Set<string>;
  steamFound: boolean;
  overlay: string;
  strings: Strings;
  onToggle: (id: string, checked: boolean) => void;
  onToggleFavorite: (id: string) => void;
  onContextMenu: (game: GameEntry, x: number, y: number) => void;
}) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: games.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 12,
  });

  return (
    <div ref={parentRef} className="game-list">
      <div className="game-list-inner" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((item) => {
          const game = games[item.index];
          return (
            <div
              key={game.id}
              className="row-slot"
              style={{ height: item.size, transform: `translateY(${item.start}px)` }}
            >
              <GameRow
                game={game}
                selected={selectedIds.has(game.id)}
                running={runningIds.has(game.id)}
                detection={detections.get(game.id)}
                favorite={favoriteIds.has(game.id)}
                disabled={game.method === "steam" && !steamFound}
                strings={strings}
                onToggle={onToggle}
                onToggleFavorite={onToggleFavorite}
                onContextMenu={onContextMenu}
              />
            </div>
          );
        })}
      </div>
      {overlay && <div className="status-overlay">{overlay}</div>}
    </div>
  );
}

const GameRow = memo(function GameRow({
  game,
  selected,
  running,
  detection,
  favorite,
  disabled,
  strings,
  onToggle,
  onToggleFavorite,
  onContextMenu,
}: {
  game: GameEntry;
  selected: boolean;
  running: boolean;
  detection: Detection | undefined;
  favorite: boolean;
  disabled: boolean;
  strings: Strings;
  onToggle: (id: string, checked: boolean) => void;
  onToggleFavorite: (id: string) => void;
  onContextMenu: (game: GameEntry, x: number, y: number) => void;
}) {
  const tooltip = game.aliases.length > 0 ? `${game.name}\n${game.aliases.join(" / ")}` : game.name;
  return (
    <div
      className={"game-row" + (selected ? " selected" : "") + (disabled ? " disabled" : "")}
      onContextMenu={(e) => {
        e.preventDefault();
        onContextMenu(game, e.clientX, e.clientY);
      }}
    >
      <button
        className={"checkbox" + (selected ? " checked" : "")}
        onClick={() => onToggle(game.id, !selected)}
        disabled={disabled}
        aria-pressed={selected}
      >
        {selected ? ICON_CHECKED : ICON_UNCHECKED}
      </button>
      <button
        className={"fav-button" + (favorite ? " on" : "")}
        onClick={() => onToggleFavorite(game.id)}
        title={favorite ? strings.RemoveFavorite : strings.AddFavorite}
        aria-pressed={favorite}
      >
        {favorite ? ICON_STAR_FILLED : ICON_STAR}
      </button>
      <div className="game-name" title={tooltip}>
        {running && <span className="idling-icon">{ICON_RUNNING}</span>}
        {running && detection === "detected" && (
          <span className="detection-icon detected" title={strings.Detected}>
            {ICON_DETECTED}
          </span>
        )}
        {running && detection === "notDetected" && (
          <span className="detection-icon not-detected" title={strings.NotDetected}>
            {ICON_NOT_DETECTED}
          </span>
        )}
        <span className="name-text">{game.name}</span>
      </div>
      <span className={"method-badge " + game.method}>{game.method === "exe" ? "EXE" : "STEAM"}</span>
      <span className="app-id">{game.steamAppId ?? ""}</span>
    </div>
  );
});
