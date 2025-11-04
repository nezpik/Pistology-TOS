import { Request, Response, NextFunction } from 'express';

// Simple in-memory cache
const cache = new Map<string, { data: any; timestamp: number }>();

// Cache duration in milliseconds (default: 5 minutes)
const CACHE_DURATION = 5 * 60 * 1000;

/**
 * Cache middleware for GET requests
 * @param duration Cache duration in milliseconds (optional)
 */
export const cacheMiddleware = (duration: number = CACHE_DURATION) => {
  return (req: Request, res: Response, next: NextFunction) => {
    // Only cache GET requests
    if (req.method !== 'GET') {
      return next();
    }

    const key = `${req.originalUrl || req.url}`;
    const cachedResponse = cache.get(key);

    if (cachedResponse) {
      const age = Date.now() - cachedResponse.timestamp;

      // Check if cache is still valid
      if (age < duration) {
        console.log(`[Cache HIT] ${key}`);
        return res.json(cachedResponse.data);
      } else {
        // Cache expired, remove it
        cache.delete(key);
      }
    }

    // Store original res.json function
    const originalJson = res.json.bind(res);

    // Override res.json to cache the response
    res.json = (data: any) => {
      console.log(`[Cache MISS] ${key}`);
      cache.set(key, {
        data,
        timestamp: Date.now()
      });
      return originalJson(data);
    };

    next();
  };
};

/**
 * Clear all cache
 */
export const clearCache = () => {
  cache.clear();
  console.log('[Cache] All cache cleared');
};

/**
 * Clear cache for specific pattern
 */
export const clearCachePattern = (pattern: string) => {
  let cleared = 0;
  for (const key of cache.keys()) {
    if (key.includes(pattern)) {
      cache.delete(key);
      cleared++;
    }
  }
  console.log(`[Cache] Cleared ${cleared} entries matching pattern: ${pattern}`);
};

/**
 * Get cache statistics
 */
export const getCacheStats = () => {
  return {
    size: cache.size,
    entries: Array.from(cache.entries()).map(([key, value]) => ({
      key,
      age: Date.now() - value.timestamp
    }))
  };
};

// Auto-cleanup expired entries every 10 minutes
setInterval(() => {
  const now = Date.now();
  let cleaned = 0;

  for (const [key, value] of cache.entries()) {
    if (now - value.timestamp > CACHE_DURATION) {
      cache.delete(key);
      cleaned++;
    }
  }

  if (cleaned > 0) {
    console.log(`[Cache] Auto-cleaned ${cleaned} expired entries`);
  }
}, 10 * 60 * 1000);
