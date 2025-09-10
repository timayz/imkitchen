package tests

import (
	"context"
	"testing"
	"time"

	"github.com/go-redis/redis/v8"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCacheService_BasicOperations(t *testing.T) {
	// Setup in-memory Redis for testing
	client := redis.NewClient(&redis.Options{
		Addr: "localhost:6379",
		DB:   15, // Use separate test DB
	})

	// Skip test if Redis not available
	ctx := context.Background()
	if err := client.Ping(ctx).Err(); err != nil {
		t.Skip("Redis not available for testing")
	}

	// Clean up after test
	defer func() {
		client.FlushDB(ctx)
		client.Close()
	}()

	service := services.NewCacheService(client)

	t.Run("Set and Get operations", func(t *testing.T) {
		key := "test:key"
		value := "test_value"
		
		// Set value
		err := service.Set(ctx, key, value, time.Minute)
		require.NoError(t, err)
		
		// Get value
		retrieved, err := service.Get(ctx, key)
		require.NoError(t, err)
		assert.Equal(t, value, retrieved)
	})

	t.Run("Rate limiting functionality", func(t *testing.T) {
		key := "test:rate_limit"
		limit := 2
		window := time.Minute

		// First request should be allowed
		allowed, remaining, err := service.CheckRateLimit(ctx, key, limit, window)
		require.NoError(t, err)
		assert.True(t, allowed)
		assert.Equal(t, 1, remaining)

		// Second request should be allowed
		allowed, remaining, err = service.CheckRateLimit(ctx, key, limit, window)
		require.NoError(t, err)
		assert.True(t, allowed)
		assert.Equal(t, 0, remaining)

		// Third request should be denied
		allowed, remaining, err = service.CheckRateLimit(ctx, key, limit, window)
		require.NoError(t, err)
		assert.False(t, allowed)
		assert.Equal(t, 0, remaining)
	})

	t.Run("Ping connectivity", func(t *testing.T) {
		err := service.Ping(ctx)
		assert.NoError(t, err)
	})
}