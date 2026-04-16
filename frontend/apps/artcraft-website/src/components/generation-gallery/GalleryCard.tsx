import { memo, useCallback, useEffect, useRef, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faCube,
  faImage,
  faSpinnerThird,
  faVideo,
} from "@fortawesome/pro-solid-svg-icons";
import { PLACEHOLDER_IMAGES } from "@storyteller/common";
import {
  getModelCreatorIconPath,
  getModelDisplayName,
} from "../../lib/omni-gen-hooks";
import type { GalleryItem } from "./useGalleryData";

// ── Persistent aspect ratio cache ─────────────────────────────────────────

const STORAGE_KEY = "gallery-aspect-ratios";

// Cap ratio so tall portraits don't dominate — 1.4 ≈ 5:7
const MAX_RATIO = 1.4;

function loadCache(): Map<string, number> {
  const map = new Map<string, number>();
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Record<string, number>;
      for (const [k, v] of Object.entries(parsed)) {
        map.set(k, v);
      }
    }
  } catch {
    // ignore
  }
  return map;
}

let persistTimer: ReturnType<typeof setTimeout> | null = null;

function persistCache(cache: Map<string, number>) {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    try {
      const entries = [...cache.entries()];
      const trimmed = entries.slice(-500);
      sessionStorage.setItem(
        STORAGE_KEY,
        JSON.stringify(Object.fromEntries(trimmed)),
      );
    } catch {
      // ignore
    }
  }, 1000);
}

export const aspectRatioCache = loadCache();

// ── Shared visibility listener for video thumbnail retries ────────────────

const visibilityCallbacks = new Set<() => void>();

function onTabVisible(cb: () => void) {
  visibilityCallbacks.add(cb);
  if (visibilityCallbacks.size === 1) {
    document.addEventListener("visibilitychange", fireVisibilityCallbacks);
  }
  return () => {
    visibilityCallbacks.delete(cb);
    if (visibilityCallbacks.size === 0) {
      document.removeEventListener(
        "visibilitychange",
        fireVisibilityCallbacks,
      );
    }
  };
}

function fireVisibilityCallbacks() {
  if (document.hidden) return;
  visibilityCallbacks.forEach((cb) => cb());
}

// ── Video thumbnail retry constants ───────────────────────────────────────

const MAX_RETRIES = 20;
const RETRY_INTERVAL = 5000;

// ── Component ──────────────────────────────────────────────────────────────

interface GalleryCardProps {
  item: GalleryItem;
  onClick: (item: GalleryItem) => void;
}

export const GalleryCard = memo(function GalleryCard({
  item,
  onClick,
}: GalleryCardProps) {
  const cached = aspectRatioCache.get(item.id);
  const [ratio, setRatio] = useState<number | undefined>(cached);
  const isVideo = item.mediaClass === "video";

  // Video thumbnail retry — "retrying" flips to true only after the first
  // error, so videos with ready thumbnails render the normal <img> path
  // with zero overhead. While retrying, a hidden <img> loads via ref
  // (no re-renders) and the spinner stays stable until success.
  const [retrying, setRetrying] = useState(false);
  const retryImgRef = useRef<HTMLImageElement>(null);
  const retryTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const retryCountRef = useRef(0);

  const kickRetry = useCallback(() => {
    if (!retryImgRef.current || !item.thumbnail) return;
    retryImgRef.current.src = `${item.thumbnail}?_r=${Date.now()}`;
  }, [item.thumbnail]);

  const scheduleRetry = useCallback(() => {
    if (retryCountRef.current >= MAX_RETRIES || !item.thumbnail) return;
    if (document.hidden) return;
    retryTimerRef.current = setTimeout(() => {
      retryCountRef.current++;
      kickRetry();
    }, RETRY_INTERVAL);
  }, [item.thumbnail, kickRetry]);

  // Subscribe to shared visibility listener for retrying video cards
  useEffect(() => {
    if (!isVideo || !item.thumbnail || !retrying) return;
    const unsubscribe = onTabVisible(() => {
      retryCountRef.current = 0;
      kickRetry();
    });
    return () => {
      unsubscribe();
      if (retryTimerRef.current) clearTimeout(retryTimerRef.current);
    };
  }, [isVideo, item.thumbnail, retrying, kickRetry]);

  // Reset retry state when thumbnail URL changes
  useEffect(() => {
    if (!isVideo) return;
    setRetrying(false);
    retryCountRef.current = 0;
    if (retryTimerRef.current) clearTimeout(retryTimerRef.current);
  }, [isVideo, item.thumbnail]);

  const displayRatio = ratio ? Math.min(ratio, MAX_RATIO) : 1;

  const style: React.CSSProperties = {
    aspectRatio: `1 / ${displayRatio}`,
    contentVisibility: "auto",
    containIntrinsicSize: "auto 200px",
  };

  const modelDisplayName = item.modelId
    ? getModelDisplayName(item.modelId)
    : null;
  const modelIconPath = item.modelId
    ? getModelCreatorIconPath(item.modelId)
    : null;

  const mediaIcon =
    item.mediaClass === "video"
      ? faVideo
      : item.mediaClass === "dimensional"
        ? faCube
        : faImage;
  const mediaLabel =
    item.mediaClass === "video"
      ? "Video"
      : item.mediaClass === "dimensional"
        ? "3D"
        : "Image";

  const handleLoad = useCallback(
    (e: React.SyntheticEvent<HTMLImageElement>) => {
      if (isVideo && retrying) {
        setRetrying(false);
        retryCountRef.current = 0;
        if (retryTimerRef.current) clearTimeout(retryTimerRef.current);
      }
      if (cached != null) return;
      const img = e.currentTarget;
      if (img.naturalWidth > 0 && img.naturalHeight > 0) {
        const r = img.naturalHeight / img.naturalWidth;
        aspectRatioCache.set(item.id, r);
        persistCache(aspectRatioCache);
        setRatio(r);
      }
    },
    [isVideo, retrying, cached, item.id],
  );

  const handleError = useCallback(
    (e: React.SyntheticEvent<HTMLImageElement>) => {
      if (isVideo) {
        setRetrying(true);
        scheduleRetry();
      } else {
        const target = e.currentTarget;
        if (target.dataset.fallback) return;
        target.dataset.fallback = "1";
        target.src = PLACEHOLDER_IMAGES.DEFAULT;
        target.style.opacity = "0.3";
      }
    },
    [isVideo, scheduleRetry],
  );

  return (
    <button
      className="group relative block w-full overflow-hidden rounded-lg bg-ui-controls/40 leading-none transition-shadow hover:ring-2 hover:ring-primary-400/60 focus:outline-none focus:ring-2 focus:ring-primary-400"
      style={style}
      onClick={() => onClick(item)}
    >
      {retrying ? (
        <>
          <div className="flex h-full w-full flex-col items-center justify-center gap-2">
            <FontAwesomeIcon
              icon={faSpinnerThird}
              className="animate-spin text-lg text-white/30"
            />
            <span className="text-[10px] text-white/30">
              Loading thumbnail…
            </span>
          </div>
          {/* Hidden img retries in background via ref — zero re-renders */}
          <img
            ref={retryImgRef}
            src={item.thumbnail!}
            alt=""
            className="absolute h-0 w-0 opacity-0"
            aria-hidden
            onLoad={handleLoad}
            onError={handleError}
          />
        </>
      ) : item.thumbnail ? (
        <img
          src={item.thumbnail}
          alt={item.label}
          loading="lazy"
          decoding="async"
          className="block h-full w-full object-cover"
          onLoad={handleLoad}
          onError={handleError}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center">
          <FontAwesomeIcon icon={mediaIcon} className="text-xl text-white/20" />
        </div>
      )}

      {/* Hover overlay with media type + model badges */}
      <div className="absolute inset-x-0 bottom-0 flex items-center gap-1.5 bg-gradient-to-t from-black/60 to-transparent px-2 pb-2 pt-6 opacity-0 transition-opacity group-hover:opacity-100">
        <div className="flex items-center gap-1 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white/80">
          <FontAwesomeIcon icon={mediaIcon} className="text-[8px]" />
          {mediaLabel}
        </div>
        {modelDisplayName && modelIconPath && (
          <div className="flex items-center gap-1 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white/80">
            <img
              src={modelIconPath}
              alt=""
              className="h-3 w-3 icon-auto-contrast"
            />
            <span className="max-w-[100px] truncate">{modelDisplayName}</span>
          </div>
        )}
      </div>
    </button>
  );
});
