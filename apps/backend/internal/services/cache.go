package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/go-redis/redis/v8"
)

// CacheService provides Redis-based caching functionality
type CacheService struct {
	client     *redis.Client
	defaultTTL time.Duration
}

// NewCacheService creates a new cache service with Redis client
func NewCacheService(client *redis.Client) *CacheService {
	return &CacheService{
		client:     client,
		defaultTTL: time.Hour,
	}
}

// Get retrieves a value from cache
func (c *CacheService) Get(ctx context.Context, key string) (string, error) {
	return c.client.Get(ctx, key).Result()
}

// Set stores a value in cache with default TTL
func (c *CacheService) Set(ctx context.Context, key string, value interface{}, ttl time.Duration) error {
	if ttl == 0 {
		ttl = c.defaultTTL
	}
	
	var data []byte
	var err error
	
	switch v := value.(type) {
	case string:
		data = []byte(v)
	case []byte:
		data = v
	default:
		data, err = json.Marshal(value)
		if err != nil {
			return fmt.Errorf("failed to marshal value: %w", err)
		}
	}
	
	return c.client.Set(ctx, key, data, ttl).Err()
}

// Delete removes a key from cache
func (c *CacheService) Delete(ctx context.Context, key string) error {
	return c.client.Del(ctx, key).Err()
}

// GetOrSet retrieves a value from cache, or sets it using the provided function
func (c *CacheService) GetOrSet(ctx context.Context, key string, ttl time.Duration, generator func() (interface{}, error)) ([]byte, error) {
	// Try to get from cache first
	cached, err := c.client.Get(ctx, key).Result()
	if err == nil {
		log.Printf("Cache hit for key: %s", key)
		return []byte(cached), nil
	}
	
	if err != redis.Nil {
		// Actual error, not just cache miss
		log.Printf("Cache error for key %s: %v", key, err)
	} else {
		log.Printf("Cache miss for key: %s", key)
	}
	
	// Cache miss - generate new value
	value, err := generator()
	if err != nil {
		return nil, err
	}
	
	// Store in cache asynchronously to not block response
	go func() {
		if err := c.Set(context.Background(), key, value, ttl); err != nil {
			log.Printf("Failed to set cache for key %s: %v", key, err)
		}
	}()
	
	// Return the generated value
	data, err := json.Marshal(value)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal generated value: %w", err)
	}
	
	return data, nil
}

// CacheRecipeSearch caches recipe search results with multi-level strategy
func (c *CacheService) CacheRecipeSearch(ctx context.Context, searchKey string, recipes interface{}, ttl time.Duration) error {
	data, err := json.Marshal(recipes)
	if err != nil {
		log.Printf("Failed to marshal recipes for caching: %v", err)
		return err
	}
	
	// Use pipeline for better performance
	pipe := c.client.Pipeline()
	
	// Cache full results
	pipe.Set(ctx, searchKey, data, ttl)
	
	// TODO: Cache individual recipes for faster lookups
	// This would require extracting individual recipes from the results
	
	_, err = pipe.Exec(ctx)
	if err != nil {
		log.Printf("Failed to execute cache pipeline: %v", err)
		return err
	}
	
	return nil
}

// CheckRateLimit validates if a request is within rate limit bounds
func (c *CacheService) CheckRateLimit(ctx context.Context, key string, limit int, window time.Duration) (bool, int, error) {
	// Use Redis pipeline for atomic operations
	pipe := c.client.Pipeline()
	incrCmd := pipe.Incr(ctx, key)
	pipe.Expire(ctx, key, window)
	
	_, err := pipe.Exec(ctx)
	if err != nil {
		log.Printf("Rate limit check failed for key %s: %v", key, err)
		// Fail open - allow request if Redis is down
		return true, 0, nil
	}
	
	current := incrCmd.Val()
	remaining := limit - int(current)
	if remaining < 0 {
		remaining = 0
	}
	
	allowed := current <= int64(limit)
	log.Printf("Rate limit check for key %s: %d/%d (allowed: %t)", key, current, limit, allowed)
	
	return allowed, remaining, nil
}

// SetupRateLimit configures rate limiting using Redis (deprecated - use CheckRateLimit)
func (c *CacheService) SetupRateLimit(ctx context.Context, key string, limit int, window time.Duration) error {
	allowed, _, err := c.CheckRateLimit(ctx, key, limit, window)
	if err != nil {
		return err
	}
	
	if !allowed {
		return fmt.Errorf("rate limit exceeded")
	}
	
	return nil
}

// Ping checks if Redis connection is healthy
func (c *CacheService) Ping(ctx context.Context) error {
	return c.client.Ping(ctx).Err()
}