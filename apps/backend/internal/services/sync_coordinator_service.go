/**
 * Sync Coordinator Service (Go Backend)
 * 
 * Coordinates sync operations across multiple clients, manages conflicts,
 * and ensures data consistency with intelligent scheduling and resource management.
 * 
 * Features:
 * - Multi-client sync coordination
 * - Conflict detection and resolution
 * - Load balancing and resource management
 * - Delta sync optimization
 * - Client prioritization and fairness
 */

package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/go-redis/redis/v8"
	"github.com/imkitchen/backend/internal/utils"
)

// SyncCoordinator manages sync operations across clients
type SyncCoordinator struct {
	redisClient       *redis.Client
	backgroundSync    *BackgroundSyncService
	activeClients     sync.Map // map[clientID]*ClientInfo
	syncSessions      sync.Map // map[sessionID]*SyncSession
	coordinatorConfig CoordinatorConfig
	mu                sync.RWMutex
}

// ClientInfo tracks information about connected clients
type ClientInfo struct {
	ClientID         string    `json:"clientId"`
	UserID           string    `json:"userId"`
	LastActivity     time.Time `json:"lastActivity"`
	SyncCapabilities []string  `json:"syncCapabilities"`
	Priority         int       `json:"priority"` // 1-10, higher is better
	ActiveSyncs      int       `json:"activeSyncs"`
	NetworkQuality   string    `json:"networkQuality"` // "poor", "good", "excellent"
	DeviceType       string    `json:"deviceType"`     // "mobile", "web", "tablet"
}

// SyncSession represents an active sync coordination session
type SyncSession struct {
	SessionID     string                 `json:"sessionId"`
	UserID        string                 `json:"userId"`
	ClientID      string                 `json:"clientId"`
	StartTime     time.Time              `json:"startTime"`
	LastUpdate    time.Time              `json:"lastUpdate"`
	SyncItems     []*SyncItem            `json:"syncItems"`
	ConflictItems []*ConflictItem        `json:"conflictItems"`
	Status        SyncSessionStatus      `json:"status"`
	Metadata      map[string]interface{} `json:"metadata"`
}

// ConflictItem represents a sync conflict that needs resolution
type ConflictItem struct {
	ItemID       string                 `json:"itemId"`
	Type         SyncItemType           `json:"type"`
	ClientData   map[string]interface{} `json:"clientData"`
	ServerData   map[string]interface{} `json:"serverData"`
	ConflictType SyncConflictType           `json:"conflictType"`
	DetectedAt   time.Time              `json:"detectedAt"`
	Severity     ConflictSeverity       `json:"severity"`
}

// ConflictType represents different types of conflicts
type SyncConflictType string

const (
	SyncConflictTypeModification SyncConflictType = "modification"
	SyncConflictTypeDeletion     SyncConflictType = "deletion"
	SyncConflictTypeCreation     SyncConflictType = "creation"
	SyncConflictTypeVersion      SyncConflictType = "version"
)

// ConflictSeverity represents conflict severity levels
type ConflictSeverity string

const (
	ConflictSeverityLow    ConflictSeverity = "low"
	ConflictSeverityMedium ConflictSeverity = "medium"
	ConflictSeverityHigh   ConflictSeverity = "high"
)

// SyncSessionStatus represents the status of a sync session
type SyncSessionStatus string

const (
	SessionStatusActive     SyncSessionStatus = "active"
	SessionStatusPaused     SyncSessionStatus = "paused"
	SessionStatusCompleted  SyncSessionStatus = "completed"
	SessionStatusConflicted SyncSessionStatus = "conflicted"
	SessionStatusError      SyncSessionStatus = "error"
)

// CoordinatorConfig holds coordinator configuration
type CoordinatorConfig struct {
	MaxConcurrentSessions    int           `json:"maxConcurrentSessions"`
	SessionTimeout           time.Duration `json:"sessionTimeout"`
	ConflictDetectionEnabled bool          `json:"conflictDetectionEnabled"`
	AutoResolutionEnabled    bool          `json:"autoResolutionEnabled"`
	ClientPriorityEnabled    bool          `json:"clientPriorityEnabled"`
	LoadBalancingEnabled     bool          `json:"loadBalancingEnabled"`
	DeltaSyncEnabled         bool          `json:"deltaSyncEnabled"`
	NetworkAdaptiveSync      bool          `json:"networkAdaptiveSync"`
}

// NewSyncCoordinator creates a new sync coordinator
func NewSyncCoordinator(redisClient *redis.Client, backgroundSync *BackgroundSyncService) *SyncCoordinator {
	config := CoordinatorConfig{
		MaxConcurrentSessions:    100,
		SessionTimeout:           30 * time.Minute,
		ConflictDetectionEnabled: true,
		AutoResolutionEnabled:    true,
		ClientPriorityEnabled:    true,
		LoadBalancingEnabled:     true,
		DeltaSyncEnabled:         true,
		NetworkAdaptiveSync:      true,
	}

	coordinator := &SyncCoordinator{
		redisClient:       redisClient,
		backgroundSync:    backgroundSync,
		coordinatorConfig: config,
	}

	coordinator.initializeCoordinator()
	return coordinator
}

// initializeCoordinator initializes the sync coordinator
func (c *SyncCoordinator) initializeCoordinator() {
	log.Println("[SyncCoordinator] Initializing sync coordinator...")

	// Start session cleanup
	go c.cleanupExpiredSessions()

	// Start load balancing
	if c.coordinatorConfig.LoadBalancingEnabled {
		go c.performLoadBalancing()
	}

	log.Println("[SyncCoordinator] Sync coordinator initialized")
}

// RegisterClient registers a new client for sync coordination
func (c *SyncCoordinator) RegisterClient(ctx context.Context, client *ClientInfo) error {
	client.LastActivity = time.Now()
	c.activeClients.Store(client.ClientID, client)

	// Store client info in Redis for persistence
	clientJSON, err := json.Marshal(client)
	if err != nil {
		return fmt.Errorf("failed to marshal client info: %w", err)
	}

	clientKey := c.getClientKey(client.ClientID)
	err = c.redisClient.Set(ctx, clientKey, clientJSON, c.coordinatorConfig.SessionTimeout).Err()
	if err != nil {
		return fmt.Errorf("failed to persist client info: %w", err)
	}

	log.Printf("[SyncCoordinator] Registered client %s for user %s (Device: %s, Network: %s)",
		client.ClientID, client.UserID, client.DeviceType, client.NetworkQuality)

	return nil
}

// StartSyncSession initiates a new sync session
func (c *SyncCoordinator) StartSyncSession(ctx context.Context, userID, clientID string, items []*SyncItem) (*SyncSession, error) {
	// Validate client
	clientInfo, exists := c.activeClients.Load(clientID)
	if !exists {
		return nil, fmt.Errorf("client %s not registered", clientID)
	}

	client := clientInfo.(*ClientInfo)
	
	// Create sync session
	session := &SyncSession{
		SessionID:     c.generateSessionID(userID, clientID),
		UserID:        userID,
		ClientID:      clientID,
		StartTime:     time.Now(),
		LastUpdate:    time.Now(),
		SyncItems:     items,
		ConflictItems: []*ConflictItem{},
		Status:        SessionStatusActive,
		Metadata:      make(map[string]interface{}),
	}

	// Detect conflicts if enabled
	if c.coordinatorConfig.ConflictDetectionEnabled {
		conflicts := c.detectConflicts(ctx, session)
		session.ConflictItems = conflicts
		
		if len(conflicts) > 0 {
			session.Status = SessionStatusConflicted
			log.Printf("[SyncCoordinator] Detected %d conflicts in session %s", len(conflicts), session.SessionID)
		}
	}

	// Store session
	c.syncSessions.Store(session.SessionID, session)
	err := c.persistSession(ctx, session)
	if err != nil {
		return nil, fmt.Errorf("failed to persist session: %w", err)
	}

	// Update client activity
	client.LastActivity = time.Now()
	client.ActiveSyncs++
	c.activeClients.Store(clientID, client)

	log.Printf("[SyncCoordinator] Started sync session %s for user %s (Items: %d, Conflicts: %d)",
		session.SessionID, userID, len(items), len(session.ConflictItems))

	return session, nil
}

// CoordinateSync coordinates sync operations with conflict resolution
func (c *SyncCoordinator) CoordinateSync(ctx context.Context, sessionID string) error {
	sessionInfo, exists := c.syncSessions.Load(sessionID)
	if !exists {
		return fmt.Errorf("session %s not found", sessionID)
	}

	session := sessionInfo.(*SyncSession)
	
	// Check for conflicts first
	if session.Status == SessionStatusConflicted {
		if c.coordinatorConfig.AutoResolutionEnabled {
			err := c.autoResolveConflicts(ctx, session)
			if err != nil {
				log.Printf("[SyncCoordinator] Auto-resolution failed for session %s: %v", sessionID, err)
				return err
			}
		} else {
			return fmt.Errorf("session %s has unresolved conflicts", sessionID)
		}
	}

	// Process sync items with coordination
	return c.processSyncItems(ctx, session)
}

// detectConflicts detects potential conflicts in sync items
func (c *SyncCoordinator) detectConflicts(ctx context.Context, session *SyncSession) []*ConflictItem {
	var conflicts []*ConflictItem

	for _, item := range session.SyncItems {
		// Check for server-side changes since client's last sync
		serverVersion := c.getServerVersion(ctx, item.ID, item.Type)
		if serverVersion > item.Version {
			// Potential conflict - fetch server data
			serverData := c.getServerData(ctx, item.ID, item.Type)
			
			conflict := &ConflictItem{
				ItemID:       item.ID,
				Type:         item.Type,
				ClientData:   item.Data,
				ServerData:   serverData,
				ConflictType: SyncConflictTypeModification,
				DetectedAt:   time.Now(),
				Severity:     c.calculateConflictSeverity(item.Data, serverData),
			}
			
			conflicts = append(conflicts, conflict)
		}
	}

	return conflicts
}

// autoResolveConflicts attempts to automatically resolve conflicts
func (c *SyncCoordinator) autoResolveConflicts(ctx context.Context, session *SyncSession) error {
	resolvedCount := 0
	
	for i, conflict := range session.ConflictItems {
		resolution := c.resolveConflict(conflict)
		
		if resolution != nil {
			// Apply resolution to sync item
			for j, item := range session.SyncItems {
				if item.ID == conflict.ItemID {
					session.SyncItems[j].Data = resolution
					break
				}
			}
			
			// Remove resolved conflict
			session.ConflictItems = append(session.ConflictItems[:i], session.ConflictItems[i+1:]...)
			resolvedCount++
		}
	}

	log.Printf("[SyncCoordinator] Auto-resolved %d/%d conflicts in session %s",
		resolvedCount, len(session.ConflictItems)+resolvedCount, session.SessionID)

	// Update session status
	if len(session.ConflictItems) == 0 {
		session.Status = SessionStatusActive
	}

	return c.persistSession(ctx, session)
}

// resolveConflict implements conflict resolution logic
func (c *SyncCoordinator) resolveConflict(conflict *ConflictItem) map[string]interface{} {
	switch conflict.Severity {
	case ConflictSeverityLow:
		// For low severity conflicts, merge automatically
		return c.mergeData(conflict.ClientData, conflict.ServerData)
		
	case ConflictSeverityMedium:
		// For medium severity, prefer client data (user's changes)
		return conflict.ClientData
		
	case ConflictSeverityHigh:
		// High severity conflicts require manual resolution
		return nil
	}
	
	return nil
}

// processSyncItems processes sync items with coordination
func (c *SyncCoordinator) processSyncItems(ctx context.Context, session *SyncSession) error {
	clientInfo, _ := c.activeClients.Load(session.ClientID)
	client := clientInfo.(*ClientInfo)

	// Adjust processing based on client capabilities
	batchSize := c.calculateOptimalBatchSize(client)
	
	// Process items in batches
	for i := 0; i < len(session.SyncItems); i += batchSize {
		end := min(i+batchSize, len(session.SyncItems))
		batch := session.SyncItems[i:end]
		
		err := c.processBatch(ctx, session, batch)
		if err != nil {
			log.Printf("[SyncCoordinator] Failed to process batch for session %s: %v", session.SessionID, err)
			session.Status = SessionStatusError
			return err
		}
		
		// Update session progress
		session.LastUpdate = time.Now()
		c.persistSession(ctx, session)
	}

	// Mark session as completed
	session.Status = SessionStatusCompleted
	session.LastUpdate = time.Now()
	
	// Update client stats
	client.ActiveSyncs--
	c.activeClients.Store(session.ClientID, client)

	log.Printf("[SyncCoordinator] Completed sync session %s", session.SessionID)
	return c.persistSession(ctx, session)
}

// processBatch processes a batch of sync items
func (c *SyncCoordinator) processBatch(ctx context.Context, session *SyncSession, batch []*SyncItem) error {
	for _, item := range batch {
		// Queue item for background processing
		err := c.backgroundSync.QueueSync(ctx, item)
		if err != nil {
			return fmt.Errorf("failed to queue sync item %s: %w", item.ID, err)
		}
	}
	
	return nil
}

// Helper methods

func (c *SyncCoordinator) generateSessionID(userID, clientID string) string {
	timestamp := time.Now().Unix()
	return fmt.Sprintf("%s_%s_%d", userID, clientID, timestamp)
}

func (c *SyncCoordinator) getClientKey(clientID string) string {
	return fmt.Sprintf("sync_client:%s", clientID)
}

func (c *SyncCoordinator) getSessionKey(sessionID string) string {
	return fmt.Sprintf("sync_session:%s", sessionID)
}

func (c *SyncCoordinator) getServerVersion(ctx context.Context, itemID string, itemType SyncItemType) int64 {
	// Mock server version lookup
	return time.Now().Unix()
}

func (c *SyncCoordinator) getServerData(ctx context.Context, itemID string, itemType SyncItemType) map[string]interface{} {
	// Mock server data lookup
	return map[string]interface{}{
		"serverModified": time.Now(),
		"version":        time.Now().Unix(),
	}
}

func (c *SyncCoordinator) calculateConflictSeverity(clientData, serverData map[string]interface{}) ConflictSeverity {
	// Simple heuristic for conflict severity
	changes := 0
	for key := range clientData {
		if clientData[key] != serverData[key] {
			changes++
		}
	}
	
	if changes > 5 {
		return ConflictSeverityHigh
	} else if changes > 2 {
		return ConflictSeverityMedium
	}
	return ConflictSeverityLow
}

func (c *SyncCoordinator) mergeData(clientData, serverData map[string]interface{}) map[string]interface{} {
	merged := make(map[string]interface{})
	
	// Start with server data
	for k, v := range serverData {
		merged[k] = v
	}
	
	// Override with client data (client wins for most fields)
	for k, v := range clientData {
		if k != "version" && k != "lastModified" {
			merged[k] = v
		}
	}
	
	return merged
}

func (c *SyncCoordinator) calculateOptimalBatchSize(client *ClientInfo) int {
	baseBatchSize := 10
	
	// Adjust based on network quality
	switch client.NetworkQuality {
	case "poor":
		return baseBatchSize / 2
	case "excellent":
		return baseBatchSize * 2
	default:
		return baseBatchSize
	}
}

func (c *SyncCoordinator) persistSession(ctx context.Context, session *SyncSession) error {
	sessionJSON, err := json.Marshal(session)
	if err != nil {
		return fmt.Errorf("failed to marshal session: %w", err)
	}

	sessionKey := c.getSessionKey(session.SessionID)
	return c.redisClient.Set(ctx, sessionKey, sessionJSON, c.coordinatorConfig.SessionTimeout).Err()
}

func (c *SyncCoordinator) cleanupExpiredSessions() {
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()

	for {
		<-ticker.C
		
		// Clean up expired sessions
		now := time.Now()
		c.syncSessions.Range(func(key, value interface{}) bool {
			session := value.(*SyncSession)
			if now.Sub(session.LastUpdate) > c.coordinatorConfig.SessionTimeout {
				c.syncSessions.Delete(key)
				log.Printf("[SyncCoordinator] Cleaned up expired session %s", session.SessionID)
			}
			return true
		})
		
		// Clean up inactive clients
		c.activeClients.Range(func(key, value interface{}) bool {
			client := value.(*ClientInfo)
			if now.Sub(client.LastActivity) > c.coordinatorConfig.SessionTimeout {
				c.activeClients.Delete(key)
				log.Printf("[SyncCoordinator] Cleaned up inactive client %s", client.ClientID)
			}
			return true
		})
	}
}

func (c *SyncCoordinator) performLoadBalancing() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for {
		<-ticker.C
		c.balanceLoad()
	}
}

func (c *SyncCoordinator) balanceLoad() {
	// Simple load balancing - adjust client priorities based on activity
	totalClients := 0
	totalActiveSyncs := 0
	
	c.activeClients.Range(func(key, value interface{}) bool {
		client := value.(*ClientInfo)
		totalClients++
		totalActiveSyncs += client.ActiveSyncs
		return true
	})
	
	if totalClients == 0 {
		return
	}
	
	avgActiveSyncs := totalActiveSyncs / totalClients
	
	c.activeClients.Range(func(key, value interface{}) bool {
		client := value.(*ClientInfo)
		
		// Adjust priority based on load
		if client.ActiveSyncs > avgActiveSyncs*2 {
			client.Priority = syncMax(1, client.Priority-1) // Lower priority for overloaded clients
		} else if client.ActiveSyncs < avgActiveSyncs/2 {
			client.Priority = utils.MinInt(10, client.Priority+1) // Higher priority for underutilized clients
		}
		
		c.activeClients.Store(key, client)
		return true
	})
}

// Public API methods

func (c *SyncCoordinator) GetActiveClients(ctx context.Context) []*ClientInfo {
	var clients []*ClientInfo
	
	c.activeClients.Range(func(key, value interface{}) bool {
		client := value.(*ClientInfo)
		clients = append(clients, client)
		return true
	})
	
	return clients
}

func (c *SyncCoordinator) GetActiveSessions(ctx context.Context, userID string) []*SyncSession {
	var sessions []*SyncSession
	
	c.syncSessions.Range(func(key, value interface{}) bool {
		session := value.(*SyncSession)
		if userID == "" || session.UserID == userID {
			sessions = append(sessions, session)
		}
		return true
	})
	
	return sessions
}

func (c *SyncCoordinator) PauseSession(ctx context.Context, sessionID string) error {
	sessionInfo, exists := c.syncSessions.Load(sessionID)
	if !exists {
		return fmt.Errorf("session %s not found", sessionID)
	}

	session := sessionInfo.(*SyncSession)
	session.Status = SessionStatusPaused
	session.LastUpdate = time.Now()
	
	return c.persistSession(ctx, session)
}

func (c *SyncCoordinator) ResumeSession(ctx context.Context, sessionID string) error {
	sessionInfo, exists := c.syncSessions.Load(sessionID)
	if !exists {
		return fmt.Errorf("session %s not found", sessionID)
	}

	session := sessionInfo.(*SyncSession)
	session.Status = SessionStatusActive
	session.LastUpdate = time.Now()
	
	return c.persistSession(ctx, session)
}

// Utility functions
func syncMax(a, b int) int {
	if a > b {
		return a
	}
	return b
}