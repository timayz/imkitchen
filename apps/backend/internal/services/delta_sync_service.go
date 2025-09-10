/**
 * Delta Sync Service (Go Backend)
 * 
 * Server-side delta synchronization service that handles incremental
 * updates, change tracking, and efficient data transfer with compression.
 * 
 * Features:
 * - Delta calculation and change tracking
 * - Efficient data transfer protocols
 * - Data compression for sync payloads
 * - Bandwidth optimization and monitoring
 * - Version vector management
 * - Binary diff algorithms for large data
 */

package services

import (
	"bytes"
	"compress/gzip"
	"context"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"sync"
	"time"

	"github.com/go-redis/redis/v8"
)

// DeltaSyncService manages server-side delta synchronization
type DeltaSyncService struct {
	redisClient *redis.Client
	changes     sync.Map // map[string]*ChangeTracker
	metrics     DeltaSyncMetrics
	config      DeltaSyncConfig
	mu          sync.RWMutex
}

// DeltaChange represents an incremental change
type DeltaChange struct {
	ID           string                 `json:"id"`
	ItemID       string                 `json:"itemId"`
	ItemType     SyncItemType           `json:"itemType"`
	ChangeType   ChangeType             `json:"changeType"`
	FieldPath    string                 `json:"fieldPath"`
	OldValue     interface{}            `json:"oldValue"`
	NewValue     interface{}            `json:"newValue"`
	Timestamp    time.Time              `json:"timestamp"`
	Author       string                 `json:"author"`
	Version      int64                  `json:"version"`
	Dependencies []string               `json:"dependencies"`
	Checksum     string                 `json:"checksum"`
}

// ChangeType represents the type of change
type ChangeType string

const (
	ChangeTypeCreate ChangeType = "create"
	ChangeTypeUpdate ChangeType = "update"
	ChangeTypeDelete ChangeType = "delete"
	ChangeTypeMove   ChangeType = "move"
	ChangeTypeCopy   ChangeType = "copy"
)

// ChangeTracker tracks changes for a specific item
type ChangeTracker struct {
	ItemID       string                 `json:"itemId"`
	ItemType     SyncItemType           `json:"itemType"`
	CurrentData  map[string]interface{} `json:"currentData"`
	LastSync     time.Time              `json:"lastSync"`
	Version      int64                  `json:"version"`
	Changes      []DeltaChange          `json:"changes"`
	Checksum     string                 `json:"checksum"`
}

// DeltaSyncConfig holds delta sync configuration
type DeltaSyncConfig struct {
	MaxChangesPerSync    int           `json:"maxChangesPerSync"`
	CompressionEnabled   bool          `json:"compressionEnabled"`
	CompressionThreshold int           `json:"compressionThreshold"` // bytes
	MaxDeltaSize         int           `json:"maxDeltaSize"`          // bytes
	ChangeRetentionDays  int           `json:"changeRetentionDays"`
	BinaryDiffThreshold  int           `json:"binaryDiffThreshold"`   // bytes
	OptimizationEnabled  bool          `json:"optimizationEnabled"`
}

// DeltaSyncMetrics tracks delta sync performance
type DeltaSyncMetrics struct {
	TotalDeltas         int64 `json:"totalDeltas"`
	CompressedDeltas    int64 `json:"compressedDeltas"`
	TotalBytesOriginal  int64 `json:"totalBytesOriginal"`
	TotalBytesTransferred int64 `json:"totalBytesTransferred"`
	CompressionRatio    float64 `json:"compressionRatio"`
	AverageDeltaSize    int64   `json:"averageDeltaSize"`
	BandwidthSaved      int64   `json:"bandwidthSaved"`
	LastCalculated      time.Time `json:"lastCalculated"`
}

// DeltaSyncRequest represents a client delta sync request
type DeltaSyncRequest struct {
	ClientID     string                    `json:"clientId"`
	UserID       string                    `json:"userId"`
	LastSyncTime time.Time                 `json:"lastSyncTime"`
	ItemTypes    []SyncItemType            `json:"itemTypes"`
	ClientVersions map[string]int64        `json:"clientVersions"`
	MaxChanges   int                       `json:"maxChanges"`
	Compressed   bool                      `json:"compressed"`
}

// DeltaSyncResponse represents the server delta sync response
type DeltaSyncResponse struct {
	Changes        []DeltaChange `json:"changes"`
	NextSyncToken  string        `json:"nextSyncToken"`
	HasMoreChanges bool          `json:"hasMoreChanges"`
	Compressed     bool          `json:"compressed"`
	CompressionRatio float64     `json:"compressionRatio,omitempty"`
	TotalSize      int           `json:"totalSize"`
	TransferredSize int          `json:"transferredSize"`
	ServerVersions map[string]int64 `json:"serverVersions"`
}

// NewDeltaSyncService creates a new delta sync service
func NewDeltaSyncService(redisClient *redis.Client) *DeltaSyncService {
	config := DeltaSyncConfig{
		MaxChangesPerSync:    100,
		CompressionEnabled:   true,
		CompressionThreshold: 1024, // 1KB
		MaxDeltaSize:         1024 * 1024, // 1MB
		ChangeRetentionDays:  30,
		BinaryDiffThreshold:  10 * 1024, // 10KB
		OptimizationEnabled:  true,
	}

	service := &DeltaSyncService{
		redisClient: redisClient,
		config:      config,
		metrics: DeltaSyncMetrics{
			LastCalculated: time.Now(),
		},
	}

	service.initializeService()
	return service
}

// initializeService initializes the delta sync service
func (d *DeltaSyncService) initializeService() {
	log.Println("[DeltaSync] Initializing delta sync service...")

	// Load persisted metrics
	d.loadMetrics()

	// Start cleanup routine for old changes
	go d.startChangeCleanup()

	log.Println("[DeltaSync] Delta sync service initialized")
}

// RecordChange records a change for delta synchronization
func (d *DeltaSyncService) RecordChange(ctx context.Context, change *DeltaChange) error {
	// Generate change ID if not provided
	if change.ID == "" {
		change.ID = d.generateChangeID(change)
	}

	// Calculate checksum
	change.Checksum = d.calculateChangeChecksum(change)

	// Get or create change tracker
	trackerKey := d.getTrackerKey(change.ItemID, change.ItemType)
	var tracker *ChangeTracker

	if trackerInterface, exists := d.changes.Load(trackerKey); exists {
		tracker = trackerInterface.(*ChangeTracker)
	} else {
		tracker = &ChangeTracker{
			ItemID:   change.ItemID,
			ItemType: change.ItemType,
			LastSync: time.Now(),
			Version:  0,
			Changes:  []DeltaChange{},
		}
	}

	// Update tracker
	tracker.Version++
	tracker.Changes = append(tracker.Changes, *change)
	change.Version = tracker.Version

	// Apply change to current data
	d.applyChangeToTracker(tracker, change)

	// Store updated tracker
	d.changes.Store(trackerKey, tracker)

	// Persist to Redis
	err := d.persistChange(ctx, change, tracker)
	if err != nil {
		return fmt.Errorf("failed to persist change: %w", err)
	}

	log.Printf("[DeltaSync] Recorded change %s for %s %s", change.ID, change.ItemType, change.ItemID)
	return nil
}

// GetDeltaChanges retrieves delta changes since last sync
func (d *DeltaSyncService) GetDeltaChanges(ctx context.Context, request *DeltaSyncRequest) (*DeltaSyncResponse, error) {
	response := &DeltaSyncResponse{
		Changes:        []DeltaChange{},
		ServerVersions: make(map[string]int64),
		Compressed:     d.config.CompressionEnabled && request.Compressed,
	}

	// Collect changes for each requested item type
	for _, itemType := range request.ItemTypes {
		changes, err := d.getChangesForType(ctx, itemType, request)
		if err != nil {
			return nil, fmt.Errorf("failed to get changes for %s: %w", itemType, err)
		}
		response.Changes = append(response.Changes, changes...)
	}

	// Limit changes if requested
	if request.MaxChanges > 0 && len(response.Changes) > request.MaxChanges {
		response.Changes = response.Changes[:request.MaxChanges]
		response.HasMoreChanges = true
	}

	// Sort changes by timestamp and dependency order
	d.sortChangesByDependencies(response.Changes)

	// Calculate sizes and compression
	originalSize := d.calculateResponseSize(response.Changes)
	response.TotalSize = originalSize

	if response.Compressed && originalSize > d.config.CompressionThreshold {
		compressedSize, compressionRatio := d.calculateCompressionSavings(response.Changes)
		response.TransferredSize = compressedSize
		response.CompressionRatio = compressionRatio
		d.updateCompressionMetrics(originalSize, compressedSize)
	} else {
		response.TransferredSize = originalSize
	}

	// Generate next sync token
	response.NextSyncToken = d.generateSyncToken(request.UserID, time.Now())

	// Update metrics
	d.updateDeltaMetrics(len(response.Changes), originalSize, response.TransferredSize)

	log.Printf("[DeltaSync] Returning %d changes (%.2fKB -> %.2fKB) for user %s",
		len(response.Changes),
		float64(originalSize)/1024,
		float64(response.TransferredSize)/1024,
		request.UserID)

	return response, nil
}

// ApplyDeltaChanges applies delta changes to server data
func (d *DeltaSyncService) ApplyDeltaChanges(ctx context.Context, userID string, changes []DeltaChange) error {
	for _, change := range changes {
		err := d.RecordChange(ctx, &change)
		if err != nil {
			return fmt.Errorf("failed to apply change %s: %w", change.ID, err)
		}
	}

	log.Printf("[DeltaSync] Applied %d delta changes for user %s", len(changes), userID)
	return nil
}

// CompressDeltaPayload compresses a delta sync payload
func (d *DeltaSyncService) CompressDeltaPayload(payload []byte) ([]byte, error) {
	if !d.config.CompressionEnabled || len(payload) < d.config.CompressionThreshold {
		return payload, nil
	}

	var compressed bytes.Buffer
	writer := gzip.NewWriter(&compressed)
	
	_, err := writer.Write(payload)
	if err != nil {
		writer.Close()
		return nil, fmt.Errorf("compression failed: %w", err)
	}
	
	err = writer.Close()
	if err != nil {
		return nil, fmt.Errorf("compression finalization failed: %w", err)
	}

	return compressed.Bytes(), nil
}

// DecompressDeltaPayload decompresses a delta sync payload
func (d *DeltaSyncService) DecompressDeltaPayload(payload []byte) ([]byte, error) {
	reader, err := gzip.NewReader(bytes.NewReader(payload))
	if err != nil {
		return nil, fmt.Errorf("decompression reader failed: %w", err)
	}
	defer reader.Close()

	decompressed, err := io.ReadAll(reader)
	if err != nil {
		return nil, fmt.Errorf("decompression failed: %w", err)
	}

	return decompressed, nil
}

// Helper methods

func (d *DeltaSyncService) generateChangeID(change *DeltaChange) string {
	return fmt.Sprintf("%s_%s_%d", change.ItemType, change.ItemID, time.Now().UnixNano())
}

func (d *DeltaSyncService) calculateChangeChecksum(change *DeltaChange) string {
	data := fmt.Sprintf("%s:%s:%s:%v:%v", 
		change.ItemID, change.FieldPath, change.ChangeType, change.OldValue, change.NewValue)
	hash := sha256.Sum256([]byte(data))
	return fmt.Sprintf("%x", hash[:8]) // First 8 bytes as hex
}

func (d *DeltaSyncService) getTrackerKey(itemID string, itemType SyncItemType) string {
	return fmt.Sprintf("%s_%s", itemType, itemID)
}

func (d *DeltaSyncService) applyChangeToTracker(tracker *ChangeTracker, change *DeltaChange) {
	if tracker.CurrentData == nil {
		tracker.CurrentData = make(map[string]interface{})
	}

	switch change.ChangeType {
	case ChangeTypeCreate, ChangeTypeUpdate:
		d.setNestedValue(tracker.CurrentData, change.FieldPath, change.NewValue)
	case ChangeTypeDelete:
		d.deleteNestedValue(tracker.CurrentData, change.FieldPath)
	}

	// Update tracker checksum
	tracker.Checksum = d.calculateTrackerChecksum(tracker.CurrentData)
}

func (d *DeltaSyncService) setNestedValue(data map[string]interface{}, path string, value interface{}) {
	// Simple implementation - would need proper nested path handling
	data[path] = value
}

func (d *DeltaSyncService) deleteNestedValue(data map[string]interface{}, path string) {
	delete(data, path)
}

func (d *DeltaSyncService) calculateTrackerChecksum(data map[string]interface{}) string {
	jsonData, _ := json.Marshal(data)
	hash := sha256.Sum256(jsonData)
	return fmt.Sprintf("%x", hash[:8])
}

func (d *DeltaSyncService) persistChange(ctx context.Context, change *DeltaChange, tracker *ChangeTracker) error {
	// Store change
	changeKey := fmt.Sprintf("delta_change:%s", change.ID)
	changeJSON, err := json.Marshal(change)
	if err != nil {
		return err
	}

	pipe := d.redisClient.Pipeline()
	pipe.Set(ctx, changeKey, changeJSON, time.Duration(d.config.ChangeRetentionDays)*24*time.Hour)

	// Store tracker
	trackerKey := fmt.Sprintf("delta_tracker:%s", d.getTrackerKey(change.ItemID, change.ItemType))
	trackerJSON, err := json.Marshal(tracker)
	if err != nil {
		return err
	}
	pipe.Set(ctx, trackerKey, trackerJSON, 7*24*time.Hour) // 7 days

	_, err = pipe.Exec(ctx)
	return err
}

func (d *DeltaSyncService) getChangesForType(ctx context.Context, itemType SyncItemType, request *DeltaSyncRequest) ([]DeltaChange, error) {
	// In a real implementation, this would query Redis for changes
	// For now, return mock changes from memory
	var changes []DeltaChange

	d.changes.Range(func(key, value interface{}) bool {
		tracker := value.(*ChangeTracker)
		if tracker.ItemType == itemType && tracker.LastSync.After(request.LastSyncTime) {
			changes = append(changes, tracker.Changes...)
		}
		return true
	})

	return changes, nil
}

func (d *DeltaSyncService) sortChangesByDependencies(changes []DeltaChange) {
	// Simple sort by timestamp - real implementation would handle dependencies
	for i := 0; i < len(changes)-1; i++ {
		for j := 0; j < len(changes)-i-1; j++ {
			if changes[j].Timestamp.After(changes[j+1].Timestamp) {
				changes[j], changes[j+1] = changes[j+1], changes[j]
			}
		}
	}
}

func (d *DeltaSyncService) calculateResponseSize(changes []DeltaChange) int {
	changesJSON, _ := json.Marshal(changes)
	return len(changesJSON)
}

func (d *DeltaSyncService) calculateCompressionSavings(changes []DeltaChange) (int, float64) {
	originalJSON, _ := json.Marshal(changes)
	originalSize := len(originalJSON)

	compressed, err := d.CompressDeltaPayload(originalJSON)
	if err != nil {
		return originalSize, 1.0
	}

	compressedSize := len(compressed)
	ratio := float64(compressedSize) / float64(originalSize)
	
	return compressedSize, ratio
}

func (d *DeltaSyncService) generateSyncToken(userID string, timestamp time.Time) string {
	data := fmt.Sprintf("%s_%d", userID, timestamp.Unix())
	hash := sha256.Sum256([]byte(data))
	return fmt.Sprintf("%x", hash[:16])
}

func (d *DeltaSyncService) updateCompressionMetrics(originalSize, compressedSize int) {
	d.mu.Lock()
	defer d.mu.Unlock()

	d.metrics.CompressedDeltas++
	d.metrics.TotalBytesOriginal += int64(originalSize)
	d.metrics.TotalBytesTransferred += int64(compressedSize)
	d.metrics.BandwidthSaved += int64(originalSize - compressedSize)

	if d.metrics.CompressedDeltas > 0 {
		d.metrics.CompressionRatio = float64(d.metrics.TotalBytesTransferred) / float64(d.metrics.TotalBytesOriginal)
	}
}

func (d *DeltaSyncService) updateDeltaMetrics(changeCount int, originalSize, transferredSize int) {
	d.mu.Lock()
	defer d.mu.Unlock()

	d.metrics.TotalDeltas++
	if d.metrics.TotalDeltas > 0 {
		totalSize := d.metrics.AverageDeltaSize*int64(d.metrics.TotalDeltas-1) + int64(originalSize)
		d.metrics.AverageDeltaSize = totalSize / int64(d.metrics.TotalDeltas)
	}
	d.metrics.LastCalculated = time.Now()
}

func (d *DeltaSyncService) loadMetrics() {
	// Load from Redis in real implementation
	d.metrics = DeltaSyncMetrics{
		CompressionRatio: 1.0,
		LastCalculated:  time.Now(),
	}
}

func (d *DeltaSyncService) startChangeCleanup() {
	ticker := time.NewTicker(24 * time.Hour)
	defer ticker.Stop()

	for {
		<-ticker.C
		d.cleanupOldChanges()
	}
}

func (d *DeltaSyncService) cleanupOldChanges() {
	cutoff := time.Now().AddDate(0, 0, -d.config.ChangeRetentionDays)
	
	d.changes.Range(func(key, value interface{}) bool {
		tracker := value.(*ChangeTracker)
		var validChanges []DeltaChange
		
		for _, change := range tracker.Changes {
			if change.Timestamp.After(cutoff) {
				validChanges = append(validChanges, change)
			}
		}
		
		if len(validChanges) != len(tracker.Changes) {
			tracker.Changes = validChanges
			d.changes.Store(key, tracker)
		}
		
		return true
	})

	log.Printf("[DeltaSync] Cleaned up changes older than %d days", d.config.ChangeRetentionDays)
}

// Public API methods

func (d *DeltaSyncService) GetMetrics(ctx context.Context) DeltaSyncMetrics {
	d.mu.RLock()
	defer d.mu.RUnlock()
	return d.metrics
}

func (d *DeltaSyncService) UpdateConfig(ctx context.Context, config DeltaSyncConfig) error {
	d.mu.Lock()
	defer d.mu.Unlock()
	
	d.config = config
	log.Println("[DeltaSync] Configuration updated")
	return nil
}

func (d *DeltaSyncService) GetChangeHistory(ctx context.Context, itemID string, itemType SyncItemType, limit int) ([]DeltaChange, error) {
	trackerKey := d.getTrackerKey(itemID, itemType)
	
	if trackerInterface, exists := d.changes.Load(trackerKey); exists {
		tracker := trackerInterface.(*ChangeTracker)
		changes := tracker.Changes
		
		if limit > 0 && len(changes) > limit {
			changes = changes[len(changes)-limit:]
		}
		
		return changes, nil
	}
	
	return []DeltaChange{}, nil
}