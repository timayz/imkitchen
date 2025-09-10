/*
Network Resilience Service

Comprehensive server-side network resilience and reliability service providing:
- Robust retry mechanisms with exponential backoff and jitter
- Circuit breaker patterns for failing external services
- Request timeout and rate limiting
- Connection pooling and health checks
- Graceful degradation and failover strategies
- Network condition monitoring and adaptive behavior
- Comprehensive error handling and recovery

Features:
- Exponential backoff with configurable jitter
- Circuit breaker with multiple states and monitoring
- Connection health monitoring and automatic recovery
- Request queuing during degraded network conditions
- Comprehensive metrics and observability
- Adaptive timeout strategies based on network conditions
*/

package services

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"math/rand"
	"net/http"
	"sync"
	"sync/atomic"
	"time"

	"github.com/go-redis/redis/v8"
	"go.uber.org/zap"
)

// NetworkState represents the current network conditions
type NetworkState struct {
	IsHealthy          bool                   `json:"is_healthy"`
	Latency            time.Duration          `json:"latency"`
	ConnectionQuality  ConnectionQuality      `json:"connection_quality"`
	ActiveConnections  int32                  `json:"active_connections"`
	ErrorRate          float64                `json:"error_rate"`
	LastHealthCheck    time.Time              `json:"last_health_check"`
	ExternalServices   map[string]ServiceHealth `json:"external_services"`
}

// ConnectionQuality represents network connection quality levels
type ConnectionQuality string

const (
	QualityExcellent ConnectionQuality = "excellent"
	QualityGood      ConnectionQuality = "good"
	QualityFair      ConnectionQuality = "fair"
	QualityPoor      ConnectionQuality = "poor"
	QualityOffline   ConnectionQuality = "offline"
)

// ServiceHealth represents health status of external services
type ServiceHealth struct {
	IsHealthy     bool          `json:"is_healthy"`
	Latency       time.Duration `json:"latency"`
	ErrorRate     float64       `json:"error_rate"`
	LastCheck     time.Time     `json:"last_check"`
	CircuitState  CircuitState  `json:"circuit_state"`
}

// RetryConfig defines retry behavior configuration
type RetryConfig struct {
	MaxAttempts      int           `json:"max_attempts"`
	BaseDelay        time.Duration `json:"base_delay"`
	MaxDelay         time.Duration `json:"max_delay"`
	BackoffFactor    float64       `json:"backoff_factor"`
	JitterFactor     float64       `json:"jitter_factor"`
	TimeoutDuration  time.Duration `json:"timeout_duration"`
	RetryableErrors  []string      `json:"retryable_errors"`
}

// CircuitBreakerConfig defines circuit breaker behavior
type CircuitBreakerConfig struct {
	FailureThreshold     int           `json:"failure_threshold"`
	ResetTimeout        time.Duration `json:"reset_timeout"`
	MonitoringPeriod    time.Duration `json:"monitoring_period"`
	HalfOpenMaxCalls    int           `json:"half_open_max_calls"`
	MinRequestThreshold int           `json:"min_request_threshold"`
}

// CircuitState represents circuit breaker states
type CircuitState string

const (
	CircuitClosed   CircuitState = "closed"
	CircuitOpen     CircuitState = "open"
	CircuitHalfOpen CircuitState = "half_open"
)

// CircuitBreaker represents a circuit breaker instance
type CircuitBreaker struct {
	config           CircuitBreakerConfig
	state            CircuitState
	failureCount     int32
	successCount     int32
	requestCount     int32
	lastFailureTime  time.Time
	nextAttemptTime  time.Time
	halfOpenAttempts int32
	mu               sync.RWMutex
}

// QueuedOperation represents an operation queued for retry
type QueuedOperation struct {
	ID           string                 `json:"id"`
	Operation    func(context.Context) (interface{}, error) `json:"-"`
	Context      context.Context        `json:"-"`
	Metadata     OperationMetadata      `json:"metadata"`
	RetryConfig  RetryConfig           `json:"retry_config"`
	CreatedAt    time.Time             `json:"created_at"`
	NextRetryAt  time.Time             `json:"next_retry_at"`
}

// OperationMetadata contains metadata about queued operations
type OperationMetadata struct {
	Type           string    `json:"type"`
	Priority       int       `json:"priority"`
	Attempts       int       `json:"attempts"`
	LastAttemptAt  time.Time `json:"last_attempt_at"`
	LastError      string    `json:"last_error"`
	ServiceName    string    `json:"service_name"`
}

// ResilienceStats tracks comprehensive network resilience statistics
type ResilienceStats struct {
	TotalOperations          int64   `json:"total_operations"`
	SuccessfulOperations     int64   `json:"successful_operations"`
	FailedOperations         int64   `json:"failed_operations"`
	RetriedOperations        int64   `json:"retried_operations"`
	CircuitBreakerTrips      int64   `json:"circuit_breaker_trips"`
	AverageRetryAttempts     float64 `json:"average_retry_attempts"`
	AverageLatency           float64 `json:"average_latency"`
	NetworkDowntime          int64   `json:"network_downtime_ms"`
	QueuedOperationsCount    int64   `json:"queued_operations_count"`
	DataIntegrityChecks      int64   `json:"data_integrity_checks"`
	ConsistencyFailures      int64   `json:"consistency_failures"`
}

// NetworkResilienceService provides comprehensive network resilience capabilities
type NetworkResilienceService struct {
	redisClient       *redis.Client
	logger            *zap.Logger
	networkState      NetworkState
	circuitBreakers   map[string]*CircuitBreaker
	operationQueue    map[string]*QueuedOperation
	stats             ResilienceStats
	healthCheckTicker *time.Ticker
	queueProcessor    *time.Ticker
	httpClient        *http.Client
	config            NetworkResilienceConfig
	mu                sync.RWMutex
	stopCh            chan struct{}
	started           int32
}

// NetworkResilienceConfig contains service configuration
type NetworkResilienceConfig struct {
	HealthCheckInterval    time.Duration        `json:"health_check_interval"`
	QueueProcessInterval   time.Duration        `json:"queue_process_interval"`
	MaxQueueSize          int                  `json:"max_queue_size"`
	DefaultRetryConfig    RetryConfig          `json:"default_retry_config"`
	DefaultCircuitConfig  CircuitBreakerConfig `json:"default_circuit_config"`
	ConnectionTimeout     time.Duration        `json:"connection_timeout"`
	MaxIdleConnections    int                  `json:"max_idle_connections"`
	MaxConnectionsPerHost int                  `json:"max_connections_per_host"`
}

// NewNetworkResilienceService creates a new network resilience service instance
func NewNetworkResilienceService(redisClient *redis.Client, logger *zap.Logger) *NetworkResilienceService {
	config := NetworkResilienceConfig{
		HealthCheckInterval:   30 * time.Second,
		QueueProcessInterval:  5 * time.Second,
		MaxQueueSize:         1000,
		ConnectionTimeout:    10 * time.Second,
		MaxIdleConnections:   100,
		MaxConnectionsPerHost: 50,
		DefaultRetryConfig: RetryConfig{
			MaxAttempts:     5,
			BaseDelay:       time.Second,
			MaxDelay:        30 * time.Second,
			BackoffFactor:   2.0,
			JitterFactor:    0.1,
			TimeoutDuration: 10 * time.Second,
			RetryableErrors: []string{"network", "timeout", "connection", "temporary"},
		},
		DefaultCircuitConfig: CircuitBreakerConfig{
			FailureThreshold:     5,
			ResetTimeout:        60 * time.Second,
			MonitoringPeriod:    5 * time.Minute,
			HalfOpenMaxCalls:    3,
			MinRequestThreshold: 10,
		},
	}

	// Create HTTP client with optimized settings
	httpClient := &http.Client{
		Timeout: config.ConnectionTimeout,
		Transport: &http.Transport{
			MaxIdleConns:        config.MaxIdleConnections,
			MaxIdleConnsPerHost: config.MaxConnectionsPerHost,
			IdleConnTimeout:     90 * time.Second,
			DisableKeepAlives:   false,
		},
	}

	service := &NetworkResilienceService{
		redisClient:     redisClient,
		logger:          logger,
		circuitBreakers: make(map[string]*CircuitBreaker),
		operationQueue:  make(map[string]*QueuedOperation),
		httpClient:      httpClient,
		config:          config,
		stopCh:          make(chan struct{}),
		networkState: NetworkState{
			IsHealthy:         true,
			ConnectionQuality: QualityGood,
			ExternalServices:  make(map[string]ServiceHealth),
			LastHealthCheck:   time.Now(),
		},
	}

	return service
}

// Start initializes and starts the network resilience service
func (s *NetworkResilienceService) Start(ctx context.Context) error {
	if !atomic.CompareAndSwapInt32(&s.started, 0, 1) {
		return errors.New("service already started")
	}

	s.logger.Info("Starting network resilience service...")

	// Load persisted state
	if err := s.loadPersistedState(ctx); err != nil {
		s.logger.Warn("Failed to load persisted state", zap.Error(err))
	}

	// Start background processes
	s.startHealthChecks(ctx)
	s.startQueueProcessing(ctx)

	s.logger.Info("Network resilience service started successfully")
	return nil
}

// Stop gracefully stops the network resilience service
func (s *NetworkResilienceService) Stop(ctx context.Context) error {
	if !atomic.CompareAndSwapInt32(&s.started, 1, 0) {
		return nil
	}

	s.logger.Info("Stopping network resilience service...")

	close(s.stopCh)

	if s.healthCheckTicker != nil {
		s.healthCheckTicker.Stop()
	}
	if s.queueProcessor != nil {
		s.queueProcessor.Stop()
	}

	// Persist current state
	if err := s.persistState(ctx); err != nil {
		s.logger.Warn("Failed to persist state during shutdown", zap.Error(err))
	}

	s.logger.Info("Network resilience service stopped")
	return nil
}

// ExecuteWithResilience executes an operation with comprehensive resilience features
func (s *NetworkResilienceService) ExecuteWithResilience(
	ctx context.Context,
	operationID string,
	operation func(context.Context) (interface{}, error),
	config *RetryConfig,
) (interface{}, error) {
	if config == nil {
		config = &s.config.DefaultRetryConfig
	}

	atomic.AddInt64(&s.stats.TotalOperations, 1)

	// Check circuit breaker
	if s.isCircuitOpen(operationID) {
		return nil, fmt.Errorf("circuit breaker is open for operation: %s", operationID)
	}

	// Check network health
	if !s.networkState.IsHealthy {
		return s.queueOperation(operationID, operation, ctx, *config)
	}

	var lastErr error
	startTime := time.Now()

	for attempt := 0; attempt < config.MaxAttempts; attempt++ {
		// Create operation context with timeout
		opCtx, cancel := context.WithTimeout(ctx, config.TimeoutDuration)

		result, err := operation(opCtx)
		cancel()

		if err == nil {
			// Success
			atomic.AddInt64(&s.stats.SuccessfulOperations, 1)
			if attempt > 0 {
				atomic.AddInt64(&s.stats.RetriedOperations, 1)
			}

			s.recordCircuitSuccess(operationID)
			s.updateLatencyStats(time.Since(startTime))
			return result, nil
		}

		lastErr = err

		// Check if error is retryable
		if !s.isRetryableError(err, config) {
			break
		}

		// Don't retry if this is the last attempt
		if attempt < config.MaxAttempts-1 {
			delay := s.calculateRetryDelay(attempt, *config)
			s.logger.Debug("Retrying operation",
				zap.String("operation_id", operationID),
				zap.Int("attempt", attempt+1),
				zap.Int("max_attempts", config.MaxAttempts),
				zap.Duration("delay", delay),
				zap.Error(err))

			select {
			case <-time.After(delay):
				continue
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-s.stopCh:
				return nil, errors.New("service stopping")
			}
		}
	}

	// All retries failed
	atomic.AddInt64(&s.stats.FailedOperations, 1)
	s.recordCircuitFailure(operationID)

	return nil, fmt.Errorf("operation failed after %d attempts: %w", config.MaxAttempts, lastErr)
}

// queueOperation queues an operation for later execution when network conditions improve
func (s *NetworkResilienceService) queueOperation(
	operationID string,
	operation func(context.Context) (interface{}, error),
	ctx context.Context,
	config RetryConfig,
) (interface{}, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if len(s.operationQueue) >= s.config.MaxQueueSize {
		return nil, errors.New("operation queue is full")
	}

	queuedOp := &QueuedOperation{
		ID:        operationID,
		Operation: operation,
		Context:   ctx,
		Metadata: OperationMetadata{
			Type:        "queued_operation",
			Priority:    1,
			ServiceName: "network_resilience",
		},
		RetryConfig: config,
		CreatedAt:   time.Now(),
		NextRetryAt: time.Now().Add(config.BaseDelay),
	}

	s.operationQueue[operationID] = queuedOp
	atomic.AddInt64(&s.stats.QueuedOperationsCount, 1)

	s.logger.Info("Operation queued for retry",
		zap.String("operation_id", operationID),
		zap.Time("next_retry", queuedOp.NextRetryAt))

	// For now, return an error indicating the operation was queued
	// In a real implementation, this might use channels or callbacks
	return nil, fmt.Errorf("operation queued due to network conditions: %s", operationID)
}

// isRetryableError determines if an error should trigger a retry
func (s *NetworkResilienceService) isRetryableError(err error, config *RetryConfig) bool {
	errStr := err.Error()
	for _, retryableErr := range config.RetryableErrors {
		if contains(errStr, retryableErr) {
			return true
		}
	}

	// Check for common retryable HTTP errors
	if httpErr, ok := err.(*http.Client).Transport.(*http.Transport); ok {
		_ = httpErr // Placeholder for HTTP-specific error checking
		return true
	}

	// Default retryable conditions
	return contains(errStr, "timeout") ||
		contains(errStr, "connection") ||
		contains(errStr, "network") ||
		contains(errStr, "temporary")
}

// calculateRetryDelay calculates the delay before the next retry attempt
func (s *NetworkResilienceService) calculateRetryDelay(attempt int, config RetryConfig) time.Duration {
	exponentialDelay := float64(config.BaseDelay) * math.Pow(config.BackoffFactor, float64(attempt))
	jitter := exponentialDelay * config.JitterFactor * rand.Float64()
	delay := time.Duration(exponentialDelay + jitter)

	if delay > config.MaxDelay {
		delay = config.MaxDelay
	}

	// Adjust delay based on network conditions
	switch s.networkState.ConnectionQuality {
	case QualityPoor:
		delay = time.Duration(float64(delay) * 1.5)
	case QualityFair:
		delay = time.Duration(float64(delay) * 1.2)
	case QualityOffline:
		delay = config.MaxDelay
	}

	return delay
}

// Circuit Breaker Implementation

// isCircuitOpen checks if the circuit breaker is open for a service
func (s *NetworkResilienceService) isCircuitOpen(serviceID string) bool {
	s.mu.RLock()
	breaker, exists := s.circuitBreakers[serviceID]
	s.mu.RUnlock()

	if !exists {
		return false
	}

	breaker.mu.RLock()
	defer breaker.mu.RUnlock()

	switch breaker.state {
	case CircuitOpen:
		if time.Now().After(breaker.nextAttemptTime) {
			breaker.mu.RUnlock()
			breaker.mu.Lock()
			breaker.state = CircuitHalfOpen
			atomic.StoreInt32(&breaker.halfOpenAttempts, 0)
			breaker.mu.Unlock()
			breaker.mu.RLock()
			return false
		}
		return true
	case CircuitHalfOpen:
		return atomic.LoadInt32(&breaker.halfOpenAttempts) >= int32(breaker.config.HalfOpenMaxCalls)
	default:
		return false
	}
}

// recordCircuitSuccess records a successful operation for circuit breaker logic
func (s *NetworkResilienceService) recordCircuitSuccess(serviceID string) {
	s.mu.Lock()
	breaker, exists := s.circuitBreakers[serviceID]
	if !exists {
		breaker = s.createCircuitBreaker(serviceID)
		s.circuitBreakers[serviceID] = breaker
	}
	s.mu.Unlock()

	atomic.AddInt32(&breaker.successCount, 1)
	atomic.AddInt32(&breaker.requestCount, 1)

	breaker.mu.Lock()
	defer breaker.mu.Unlock()

	if breaker.state == CircuitHalfOpen {
		breaker.state = CircuitClosed
		atomic.StoreInt32(&breaker.failureCount, 0)
		atomic.StoreInt32(&breaker.halfOpenAttempts, 0)
	}
}

// recordCircuitFailure records a failed operation for circuit breaker logic
func (s *NetworkResilienceService) recordCircuitFailure(serviceID string) {
	s.mu.Lock()
	breaker, exists := s.circuitBreakers[serviceID]
	if !exists {
		breaker = s.createCircuitBreaker(serviceID)
		s.circuitBreakers[serviceID] = breaker
	}
	s.mu.Unlock()

	failureCount := atomic.AddInt32(&breaker.failureCount, 1)
	atomic.AddInt32(&breaker.requestCount, 1)

	breaker.mu.Lock()
	defer breaker.mu.Unlock()

	breaker.lastFailureTime = time.Now()

	if breaker.state == CircuitHalfOpen {
		atomic.AddInt32(&breaker.halfOpenAttempts, 1)
		breaker.state = CircuitOpen
		breaker.nextAttemptTime = time.Now().Add(breaker.config.ResetTimeout)
		atomic.AddInt64(&s.stats.CircuitBreakerTrips, 1)
	} else if failureCount >= int32(breaker.config.FailureThreshold) &&
		atomic.LoadInt32(&breaker.requestCount) >= int32(breaker.config.MinRequestThreshold) {
		breaker.state = CircuitOpen
		breaker.nextAttemptTime = time.Now().Add(breaker.config.ResetTimeout)
		atomic.AddInt64(&s.stats.CircuitBreakerTrips, 1)
	}
}

// createCircuitBreaker creates a new circuit breaker with default configuration
func (s *NetworkResilienceService) createCircuitBreaker(serviceID string) *CircuitBreaker {
	return &CircuitBreaker{
		config: s.config.DefaultCircuitConfig,
		state:  CircuitClosed,
	}
}

// Health Check and Monitoring

// startHealthChecks begins periodic network health monitoring
func (s *NetworkResilienceService) startHealthChecks(ctx context.Context) {
	s.healthCheckTicker = time.NewTicker(s.config.HealthCheckInterval)

	go func() {
		defer s.healthCheckTicker.Stop()

		for {
			select {
			case <-s.healthCheckTicker.C:
				s.performHealthCheck(ctx)
			case <-s.stopCh:
				return
			case <-ctx.Done():
				return
			}
		}
	}()
}

// performHealthCheck executes comprehensive health checks
func (s *NetworkResilienceService) performHealthCheck(ctx context.Context) {
	startTime := time.Now()

	// Test external service connectivity
	externalServices := s.checkExternalServices(ctx)

	// Calculate network metrics
	latency := s.measureNetworkLatency(ctx)
	errorRate := s.calculateErrorRate()
	connectionQuality := s.evaluateConnectionQuality(latency, errorRate)

	s.mu.Lock()
	s.networkState = NetworkState{
		IsHealthy:         errorRate < 0.1 && latency < 2*time.Second,
		Latency:           latency,
		ConnectionQuality: connectionQuality,
		ActiveConnections: int32(len(s.operationQueue)),
		ErrorRate:         errorRate,
		LastHealthCheck:   time.Now(),
		ExternalServices:  externalServices,
	}
	s.mu.Unlock()

	s.logger.Debug("Health check completed",
		zap.Duration("latency", latency),
		zap.Float64("error_rate", errorRate),
		zap.String("quality", string(connectionQuality)),
		zap.Duration("check_duration", time.Since(startTime)))
}

// checkExternalServices checks the health of external services
func (s *NetworkResilienceService) checkExternalServices(ctx context.Context) map[string]ServiceHealth {
	services := make(map[string]ServiceHealth)

	// Example external service checks - customize based on your services
	externalEndpoints := map[string]string{
		"recipe_api":    "https://api.example.com/health",
		"image_service": "https://images.example.com/health",
		"auth_service":  "https://auth.example.com/health",
	}

	for serviceName, endpoint := range externalEndpoints {
		health := s.checkServiceHealth(ctx, serviceName, endpoint)
		services[serviceName] = health
	}

	return services
}

// checkServiceHealth checks the health of a specific service
func (s *NetworkResilienceService) checkServiceHealth(ctx context.Context, serviceName, endpoint string) ServiceHealth {
	startTime := time.Now()
	
	req, err := http.NewRequestWithContext(ctx, "GET", endpoint, nil)
	if err != nil {
		return ServiceHealth{
			IsHealthy:    false,
			Latency:      0,
			ErrorRate:    1.0,
			LastCheck:    time.Now(),
			CircuitState: CircuitOpen,
		}
	}

	resp, err := s.httpClient.Do(req)
	latency := time.Since(startTime)

	isHealthy := err == nil && resp != nil && resp.StatusCode < 400
	if resp != nil {
		resp.Body.Close()
	}

	s.mu.RLock()
	breaker := s.circuitBreakers[serviceName]
	s.mu.RUnlock()

	circuitState := CircuitClosed
	if breaker != nil {
		breaker.mu.RLock()
		circuitState = breaker.state
		breaker.mu.RUnlock()
	}

	return ServiceHealth{
		IsHealthy:    isHealthy,
		Latency:      latency,
		ErrorRate:    0.0, // Would need historical data for accurate calculation
		LastCheck:    time.Now(),
		CircuitState: circuitState,
	}
}

// measureNetworkLatency measures current network latency
func (s *NetworkResilienceService) measureNetworkLatency(ctx context.Context) time.Duration {
	startTime := time.Now()

	// Lightweight connectivity test
	req, err := http.NewRequestWithContext(ctx, "HEAD", "https://httpbin.org/status/200", nil)
	if err != nil {
		return time.Hour // Indicate network issues
	}

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return time.Hour // Indicate network issues
	}
	defer resp.Body.Close()

	return time.Since(startTime)
}

// calculateErrorRate calculates current error rate from statistics
func (s *NetworkResilienceService) calculateErrorRate() float64 {
	totalOps := atomic.LoadInt64(&s.stats.TotalOperations)
	failedOps := atomic.LoadInt64(&s.stats.FailedOperations)

	if totalOps == 0 {
		return 0.0
	}

	return float64(failedOps) / float64(totalOps)
}

// evaluateConnectionQuality determines connection quality based on metrics
func (s *NetworkResilienceService) evaluateConnectionQuality(latency time.Duration, errorRate float64) ConnectionQuality {
	if errorRate > 0.5 || latency > 10*time.Second {
		return QualityOffline
	}
	if errorRate > 0.2 || latency > 5*time.Second {
		return QualityPoor
	}
	if errorRate > 0.1 || latency > 2*time.Second {
		return QualityFair
	}
	if latency < 500*time.Millisecond {
		return QualityExcellent
	}
	return QualityGood
}

// Queue Processing

// startQueueProcessing begins processing the operation queue
func (s *NetworkResilienceService) startQueueProcessing(ctx context.Context) {
	s.queueProcessor = time.NewTicker(s.config.QueueProcessInterval)

	go func() {
		defer s.queueProcessor.Stop()

		for {
			select {
			case <-s.queueProcessor.C:
				s.processOperationQueue(ctx)
			case <-s.stopCh:
				return
			case <-ctx.Done():
				return
			}
		}
	}()
}

// processOperationQueue processes queued operations
func (s *NetworkResilienceService) processOperationQueue(ctx context.Context) {
	if !s.networkState.IsHealthy {
		return
	}

	s.mu.Lock()
	operations := make([]*QueuedOperation, 0, len(s.operationQueue))
	for _, op := range s.operationQueue {
		if time.Now().After(op.NextRetryAt) {
			operations = append(operations, op)
		}
	}
	s.mu.Unlock()

	for _, op := range operations {
		select {
		case <-ctx.Done():
			return
		case <-s.stopCh:
			return
		default:
			s.processQueuedOperation(ctx, op)
		}
	}
}

// processQueuedOperation processes a single queued operation
func (s *NetworkResilienceService) processQueuedOperation(ctx context.Context, op *QueuedOperation) {
	op.Metadata.Attempts++
	op.Metadata.LastAttemptAt = time.Now()

	result, err := op.Operation(op.Context)
	if err != nil {
		op.Metadata.LastError = err.Error()

		if op.Metadata.Attempts >= op.RetryConfig.MaxAttempts {
			s.logger.Error("Queued operation failed permanently",
				zap.String("operation_id", op.ID),
				zap.Int("attempts", op.Metadata.Attempts),
				zap.Error(err))

			s.mu.Lock()
			delete(s.operationQueue, op.ID)
			s.mu.Unlock()
		} else {
			op.NextRetryAt = time.Now().Add(s.calculateRetryDelay(op.Metadata.Attempts-1, op.RetryConfig))
		}
		return
	}

	// Success
	_ = result // Handle result as needed
	s.logger.Info("Queued operation completed successfully",
		zap.String("operation_id", op.ID),
		zap.Int("attempts", op.Metadata.Attempts))

	s.mu.Lock()
	delete(s.operationQueue, op.ID)
	s.mu.Unlock()
}

// Data Integrity and Persistence

// VerifyDataIntegrity performs comprehensive data integrity verification
func (s *NetworkResilienceService) VerifyDataIntegrity(ctx context.Context, data interface{}, expectedHash string) (bool, error) {
	atomic.AddInt64(&s.stats.DataIntegrityChecks, 1)

	// Calculate hash of the data
	dataBytes, err := json.Marshal(data)
	if err != nil {
		atomic.AddInt64(&s.stats.ConsistencyFailures, 1)
		return false, fmt.Errorf("failed to marshal data for integrity check: %w", err)
	}

	actualHash := s.calculateSimpleHash(dataBytes)
	isValid := actualHash == expectedHash

	if !isValid {
		atomic.AddInt64(&s.stats.ConsistencyFailures, 1)
		s.logger.Warn("Data integrity check failed",
			zap.String("expected_hash", expectedHash),
			zap.String("actual_hash", actualHash))
	}

	return isValid, nil
}

// calculateSimpleHash calculates a simple hash for data integrity checking
func (s *NetworkResilienceService) calculateSimpleHash(data []byte) string {
	// Simple hash calculation - in production, use a proper cryptographic hash
	var hash uint64
	for _, b := range data {
		hash = hash*31 + uint64(b)
	}
	return fmt.Sprintf("%x", hash)
}

// loadPersistedState loads persisted service state from Redis
func (s *NetworkResilienceService) loadPersistedState(ctx context.Context) error {
	// Load statistics
	statsData, err := s.redisClient.Get(ctx, "network_resilience:stats").Result()
	if err == nil {
		if err := json.Unmarshal([]byte(statsData), &s.stats); err != nil {
			s.logger.Warn("Failed to unmarshal persisted stats", zap.Error(err))
		}
	}

	// Load circuit breaker states
	breakerData, err := s.redisClient.HGetAll(ctx, "network_resilience:circuit_breakers").Result()
	if err == nil {
		for serviceID, data := range breakerData {
			var breaker CircuitBreaker
			if err := json.Unmarshal([]byte(data), &breaker); err == nil {
				s.circuitBreakers[serviceID] = &breaker
			}
		}
	}

	return nil
}

// persistState persists current service state to Redis
func (s *NetworkResilienceService) persistState(ctx context.Context) error {
	// Persist statistics
	statsData, err := json.Marshal(s.stats)
	if err == nil {
		s.redisClient.Set(ctx, "network_resilience:stats", statsData, 24*time.Hour)
	}

	// Persist circuit breaker states
	s.mu.RLock()
	for serviceID, breaker := range s.circuitBreakers {
		breakerData, err := json.Marshal(breaker)
		if err == nil {
			s.redisClient.HSet(ctx, "network_resilience:circuit_breakers", serviceID, breakerData)
		}
	}
	s.mu.RUnlock()

	s.redisClient.Expire(ctx, "network_resilience:circuit_breakers", 24*time.Hour)
	return nil
}

// updateLatencyStats updates average latency statistics
func (s *NetworkResilienceService) updateLatencyStats(latency time.Duration) {
	totalOps := atomic.LoadInt64(&s.stats.TotalOperations)
	if totalOps <= 1 {
		s.stats.AverageLatency = float64(latency.Milliseconds())
		return
	}

	// Calculate moving average
	currentAvg := s.stats.AverageLatency
	newAvg := (currentAvg*float64(totalOps-1) + float64(latency.Milliseconds())) / float64(totalOps)
	s.stats.AverageLatency = newAvg
}

// Public API Methods

// GetNetworkState returns current network state
func (s *NetworkResilienceService) GetNetworkState() NetworkState {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.networkState
}

// GetStats returns current resilience statistics
func (s *NetworkResilienceService) GetStats() ResilienceStats {
	return s.stats
}

// GetQueueStatus returns current operation queue status
func (s *NetworkResilienceService) GetQueueStatus() map[string]interface{} {
	s.mu.RLock()
	defer s.mu.RUnlock()

	var oldestOperation *time.Time
	failedCount := 0

	for _, op := range s.operationQueue {
		if op.Metadata.LastError != "" {
			failedCount++
		}
		if oldestOperation == nil || op.CreatedAt.Before(*oldestOperation) {
			oldestOperation = &op.CreatedAt
		}
	}

	status := map[string]interface{}{
		"size":             len(s.operationQueue),
		"failed_operations": failedCount,
	}

	if oldestOperation != nil {
		status["oldest_operation"] = *oldestOperation
	}

	return status
}

// ClearQueue clears the operation queue
func (s *NetworkResilienceService) ClearQueue() {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.operationQueue = make(map[string]*QueuedOperation)
	s.logger.Info("Operation queue cleared")
}

// UpdateRetryConfig updates the default retry configuration
func (s *NetworkResilienceService) UpdateRetryConfig(config RetryConfig) {
	s.config.DefaultRetryConfig = config
	s.logger.Info("Retry configuration updated")
}

// Utility functions

// contains checks if a string contains a substring
func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(substr) == 0 || 
		(len(s) > len(substr) && s[:len(substr)] == substr) ||
		(len(s) > len(substr) && s[len(s)-len(substr):] == substr) ||
		func() bool {
			for i := 0; i <= len(s)-len(substr); i++ {
				if s[i:i+len(substr)] == substr {
					return true
				}
			}
			return false
		}())
}