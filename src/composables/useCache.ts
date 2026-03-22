/**
 * Lightweight TTL-cache for Tauri invoke() results.
 * Prevents redundant backend calls on re-mount / navigation.
 *
 * Usage:
 *   const { cached } = useCache()
 *   const data = await cached('key', () => invoke('cmd'), 30_000)
 */

interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

const store = new Map<string, CacheEntry<unknown>>();

export function useCache() {
  async function cached<T>(
    key: string,
    fetcher: () => Promise<T>,
    ttlMs = 30_000
  ): Promise<T> {
    const now = Date.now();
    const hit = store.get(key);
    if (hit && hit.expiresAt > now) {
      return hit.value as T;
    }
    const value = await fetcher();
    store.set(key, { value, expiresAt: now + ttlMs });
    return value;
  }

  function invalidate(key: string) {
    store.delete(key);
  }

  function invalidatePrefix(prefix: string) {
    for (const k of store.keys()) {
      if (k.startsWith(prefix)) store.delete(k);
    }
  }

  return { cached, invalidate, invalidatePrefix };
}
