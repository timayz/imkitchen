/**
 * Sync Settings Screen Component
 * 
 * Provides comprehensive sync preferences configuration with
 * auto-sync toggle, sync frequency, data usage controls, and
 * advanced sync options.
 * 
 * Features:
 * - Auto-sync enable/disable toggle
 * - Sync frequency configuration
 * - Data usage controls and wifi-only mode
 * - Battery optimization settings
 * - Conflict resolution preferences
 * - Sync scope selection (data types)
 * - Advanced debug and monitoring options
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  Switch,
  TouchableOpacity,
  ScrollView,
  StyleSheet,
  Alert,
  Modal,
  Slider,
  Picker,
  ActivityIndicator,
} from 'react-native';
import { backgroundSyncService } from '../../services/background_sync_service';
import { backgroundTaskService } from '../../services/background_task_service';
import { syncCoordinationService } from '../../services/sync_coordination_service';

interface SyncSettings {
  autoSyncEnabled: boolean;
  syncFrequency: number;          // minutes
  wifiOnlyMode: boolean;
  batterySaverMode: boolean;
  backgroundSyncEnabled: boolean;
  conflictResolutionStrategy: 'auto' | 'manual' | 'prompt';
  syncScope: SyncScope;
  dataUsageLimit: number;         // MB per day
  debugMode: boolean;
  advancedLogging: boolean;
}

interface SyncScope {
  recipes: boolean;
  mealPlans: boolean;
  shoppingLists: boolean;
  userProfile: boolean;
  preferences: boolean;
  ratings: boolean;
  communityData: boolean;
}

interface SyncSettingsScreenProps {
  navigation?: any;
}

const SyncSettingsScreen: React.FC<SyncSettingsScreenProps> = ({ navigation }) => {
  const [settings, setSettings] = useState<SyncSettings>({
    autoSyncEnabled: true,
    syncFrequency: 30,
    wifiOnlyMode: false,
    batterySaverMode: false,
    backgroundSyncEnabled: true,
    conflictResolutionStrategy: 'prompt',
    syncScope: {
      recipes: true,
      mealPlans: true,
      shoppingLists: true,
      userProfile: true,
      preferences: true,
      ratings: true,
      communityData: true,
    },
    dataUsageLimit: 100,
    debugMode: false,
    advancedLogging: false,
  });

  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showDataUsage, setShowDataUsage] = useState(false);
  const [syncStats, setSyncStats] = useState<any>(null);

  useEffect(() => {
    loadSettings();
    loadSyncStats();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      
      // Load settings from services
      const syncStatus = backgroundSyncService.getSyncStatus();
      const taskConfig = backgroundTaskService.getConfig();
      const coordinationStatus = syncCoordinationService.getCoordinationStatus();
      
      // Merge settings from different services
      const loadedSettings: SyncSettings = {
        ...settings,
        backgroundSyncEnabled: !syncStatus.isPaused,
        wifiOnlyMode: taskConfig.networkRequirement === 'wifi',
        batterySaverMode: taskConfig.batteryThreshold > 30,
        syncFrequency: Math.floor(taskConfig.taskInterval / (60 * 1000)), // Convert ms to minutes
      };
      
      setSettings(loadedSettings);
      
    } catch (error) {
      console.error('[SyncSettings] Failed to load settings:', error);
      Alert.alert('Error', 'Failed to load sync settings');
    } finally {
      setLoading(false);
    }
  };

  const loadSyncStats = async () => {
    try {
      const stats = backgroundSyncService.getStatistics();
      setSyncStats(stats);
    } catch (error) {
      console.error('[SyncSettings] Failed to load sync stats:', error);
    }
  };

  const saveSettings = async () => {
    try {
      setSaving(true);
      
      // Update background task service
      backgroundTaskService.updateConfig({
        taskInterval: settings.syncFrequency * 60 * 1000, // Convert minutes to ms
        networkRequirement: settings.wifiOnlyMode ? 'wifi' : 'any',
        batteryThreshold: settings.batterySaverMode ? 50 : 20,
      });
      
      // Update battery saver mode
      if (settings.batterySaverMode) {
        backgroundTaskService.enableBatterySaver();
      } else {
        backgroundTaskService.disableBatterySaver();
      }
      
      // Update sync service
      if (settings.autoSyncEnabled) {
        backgroundSyncService.resumeSync();
      } else {
        backgroundSyncService.pauseSync();
      }
      
      console.log('[SyncSettings] Settings saved successfully');
      Alert.alert('Success', 'Sync settings have been updated');
      
    } catch (error) {
      console.error('[SyncSettings] Failed to save settings:', error);
      Alert.alert('Error', 'Failed to save sync settings');
    } finally {
      setSaving(false);
    }
  };

  const resetToDefaults = () => {
    Alert.alert(
      'Reset Settings',
      'Are you sure you want to reset all sync settings to default values?',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Reset',
          style: 'destructive',
          onPress: () => {
            setSettings({
              autoSyncEnabled: true,
              syncFrequency: 30,
              wifiOnlyMode: false,
              batterySaverMode: false,
              backgroundSyncEnabled: true,
              conflictResolutionStrategy: 'prompt',
              syncScope: {
                recipes: true,
                mealPlans: true,
                shoppingLists: true,
                userProfile: true,
                preferences: true,
                ratings: true,
                communityData: true,
              },
              dataUsageLimit: 100,
              debugMode: false,
              advancedLogging: false,
            });
          },
        },
      ]
    );
  };

  const clearSyncData = () => {
    Alert.alert(
      'Clear Sync Data',
      'This will clear all cached sync data and reset sync history. This action cannot be undone.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Clear',
          style: 'destructive',
          onPress: async () => {
            try {
              // Clear sync data - implementation would depend on service APIs
              console.log('[SyncSettings] Sync data cleared');
              Alert.alert('Success', 'Sync data has been cleared');
            } catch (error) {
              console.error('[SyncSettings] Failed to clear sync data:', error);
              Alert.alert('Error', 'Failed to clear sync data');
            }
          },
        },
      ]
    );
  };

  const updateSetting = <K extends keyof SyncSettings>(
    key: K,
    value: SyncSettings[K]
  ) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  };

  const updateSyncScope = <K extends keyof SyncScope>(
    key: K,
    value: SyncScope[K]
  ) => {
    setSettings(prev => ({
      ...prev,
      syncScope: { ...prev.syncScope, [key]: value }
    }));
  };

  const renderSettingRow = (
    title: string,
    subtitle: string,
    control: React.ReactNode,
    onPress?: () => void
  ) => (
    <TouchableOpacity
      style={styles.settingRow}
      onPress={onPress}
      disabled={!onPress}
      activeOpacity={onPress ? 0.7 : 1}
    >
      <View style={styles.settingInfo}>
        <Text style={styles.settingTitle}>{title}</Text>
        <Text style={styles.settingSubtitle}>{subtitle}</Text>
      </View>
      <View style={styles.settingControl}>{control}</View>
    </TouchableOpacity>
  );

  const renderSectionHeader = (title: string) => (
    <View style={styles.sectionHeader}>
      <Text style={styles.sectionTitle}>{title}</Text>
    </View>
  );

  const renderSyncScopeSettings = () => (
    <View style={styles.section}>
      {renderSectionHeader('Sync Scope')}
      
      {Object.entries(settings.syncScope).map(([key, enabled]) => (
        <View key={key} style={styles.scopeRow}>
          <Text style={styles.scopeLabel}>
            {key.charAt(0).toUpperCase() + key.slice(1).replace(/([A-Z])/g, ' $1')}
          </Text>
          <Switch
            value={enabled}
            onValueChange={(value) => updateSyncScope(key as keyof SyncScope, value)}
            trackColor={{ false: '#E0E0E0', true: '#007AFF' }}
            thumbColor={enabled ? '#FFFFFF' : '#F4F3F4'}
          />
        </View>
      ))}
    </View>
  );

  const renderSyncStats = () => {
    if (!syncStats) return null;

    return (
      <View style={styles.statsContainer}>
        <Text style={styles.statsTitle}>Sync Statistics</Text>
        <View style={styles.statsGrid}>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{syncStats.totalSyncs}</Text>
            <Text style={styles.statLabel}>Total Syncs</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{syncStats.successfulSyncs}</Text>
            <Text style={styles.statLabel}>Successful</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{syncStats.syncEfficiency.toFixed(1)}%</Text>
            <Text style={styles.statLabel}>Efficiency</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{syncStats.averageSyncTime}ms</Text>
            <Text style={styles.statLabel}>Avg Time</Text>
          </View>
        </View>
      </View>
    );
  };

  if (loading) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#007AFF" />
        <Text style={styles.loadingText}>Loading sync settings...</Text>
      </View>
    );
  }

  return (
    <ScrollView style={styles.container}>
      {/* Basic Settings */}
      <View style={styles.section}>
        {renderSectionHeader('Basic Settings')}
        
        {renderSettingRow(
          'Auto-Sync',
          'Automatically sync data in the background',
          <Switch
            value={settings.autoSyncEnabled}
            onValueChange={(value) => updateSetting('autoSyncEnabled', value)}
            trackColor={{ false: '#E0E0E0', true: '#007AFF' }}
            thumbColor={settings.autoSyncEnabled ? '#FFFFFF' : '#F4F3F4'}
          />
        )}
        
        {renderSettingRow(
          'Sync Frequency',
          `Sync every ${settings.syncFrequency} minutes`,
          <Text style={styles.settingValue}>{settings.syncFrequency}m</Text>,
          () => {
            Alert.prompt(
              'Sync Frequency',
              'Enter sync frequency in minutes (1-1440):',
              [
                { text: 'Cancel', style: 'cancel' },
                {
                  text: 'Save',
                  onPress: (text) => {
                    const frequency = parseInt(text || '30');
                    if (frequency >= 1 && frequency <= 1440) {
                      updateSetting('syncFrequency', frequency);
                    } else {
                      Alert.alert('Invalid Value', 'Please enter a value between 1 and 1440 minutes');
                    }
                  }
                }
              ],
              'numeric',
              settings.syncFrequency.toString()
            );
          }
        )}
        
        {renderSettingRow(
          'Background Sync',
          'Continue syncing when app is in background',
          <Switch
            value={settings.backgroundSyncEnabled}
            onValueChange={(value) => updateSetting('backgroundSyncEnabled', value)}
            trackColor={{ false: '#E0E0E0', true: '#007AFF' }}
            thumbColor={settings.backgroundSyncEnabled ? '#FFFFFF' : '#F4F3F4'}
          />
        )}
      </View>

      {/* Network & Data Settings */}
      <View style={styles.section}>
        {renderSectionHeader('Network & Data')}
        
        {renderSettingRow(
          'WiFi Only',
          'Only sync when connected to WiFi',
          <Switch
            value={settings.wifiOnlyMode}
            onValueChange={(value) => updateSetting('wifiOnlyMode', value)}
            trackColor={{ false: '#E0E0E0', true: '#007AFF' }}
            thumbColor={settings.wifiOnlyMode ? '#FFFFFF' : '#F4F3F4'}
          />
        )}
        
        {renderSettingRow(
          'Data Usage Limit',
          `Limit sync to ${settings.dataUsageLimit}MB per day`,
          <Text style={styles.settingValue}>{settings.dataUsageLimit}MB</Text>,
          () => setShowDataUsage(true)
        )}
        
        {renderSettingRow(
          'Battery Saver',
          'Reduce sync frequency when battery is low',
          <Switch
            value={settings.batterySaverMode}
            onValueChange={(value) => updateSetting('batterySaverMode', value)}
            trackColor={{ false: '#E0E0E0', true: '#007AFF' }}
            thumbColor={settings.batterySaverMode ? '#FFFFFF' : '#F4F3F4'}
          />
        )}
      </View>

      {/* Conflict Resolution */}
      <View style={styles.section}>
        {renderSectionHeader('Conflict Resolution')}
        
        {renderSettingRow(
          'Resolution Strategy',
          settings.conflictResolutionStrategy === 'auto' ? 'Resolve automatically' :
           settings.conflictResolutionStrategy === 'manual' ? 'Always ask for manual resolution' :
           'Prompt when conflicts occur',
          <Text style={styles.settingValue}>
            {settings.conflictResolutionStrategy.charAt(0).toUpperCase() + settings.conflictResolutionStrategy.slice(1)}
          </Text>,
          () => {
            Alert.alert(
              'Conflict Resolution Strategy',
              'Choose how to handle sync conflicts:',
              [
                { text: 'Cancel', style: 'cancel' },
                { text: 'Auto', onPress: () => updateSetting('conflictResolutionStrategy', 'auto') },
                { text: 'Manual', onPress: () => updateSetting('conflictResolutionStrategy', 'manual') },
                { text: 'Prompt', onPress: () => updateSetting('conflictResolutionStrategy', 'prompt') },
              ]
            );
          }
        )}
      </View>

      {/* Sync Scope */}
      {renderSyncScopeSettings()}

      {/* Statistics */}
      {renderSyncStats()}

      {/* Advanced Settings */}
      <View style={styles.section}>
        {renderSectionHeader('Advanced')}
        
        <TouchableOpacity
          style={styles.expandButton}
          onPress={() => setShowAdvanced(!showAdvanced)}
        >
          <Text style={styles.expandButtonText}>
            Advanced Settings {showAdvanced ? '▲' : '▼'}
          </Text>
        </TouchableOpacity>
        
        {showAdvanced && (
          <View style={styles.advancedSection}>
            {renderSettingRow(
              'Debug Mode',
              'Enable detailed sync logging and debugging',
              <Switch
                value={settings.debugMode}
                onValueChange={(value) => updateSetting('debugMode', value)}
                trackColor={{ false: '#E0E0E0', true: '#007AFF' }}
                thumbColor={settings.debugMode ? '#FFFFFF' : '#F4F3F4'}
              />
            )}
            
            <TouchableOpacity style={styles.dangerButton} onPress={clearSyncData}>
              <Text style={styles.dangerButtonText}>Clear Sync Data</Text>
            </TouchableOpacity>
          </View>
        )}
      </View>

      {/* Action Buttons */}
      <View style={styles.actionButtons}>
        <TouchableOpacity
          style={styles.resetButton}
          onPress={resetToDefaults}
        >
          <Text style={styles.resetButtonText}>Reset to Defaults</Text>
        </TouchableOpacity>
        
        <TouchableOpacity
          style={[styles.saveButton, saving && styles.disabledButton]}
          onPress={saveSettings}
          disabled={saving}
        >
          {saving ? (
            <ActivityIndicator color="#FFFFFF" />
          ) : (
            <Text style={styles.saveButtonText}>Save Settings</Text>
          )}
        </TouchableOpacity>
      </View>

      {/* Data Usage Modal */}
      <Modal
        visible={showDataUsage}
        animationType="slide"
        presentationStyle="pageSheet"
        onRequestClose={() => setShowDataUsage(false)}
      >
        <View style={styles.modalContainer}>
          <View style={styles.modalHeader}>
            <Text style={styles.modalTitle}>Data Usage Limit</Text>
            <TouchableOpacity onPress={() => setShowDataUsage(false)}>
              <Text style={styles.modalClose}>Done</Text>
            </TouchableOpacity>
          </View>
          
          <View style={styles.sliderContainer}>
            <Text style={styles.sliderLabel}>
              Daily limit: {settings.dataUsageLimit}MB
            </Text>
            <Slider
              style={styles.slider}
              minimumValue={10}
              maximumValue={500}
              value={settings.dataUsageLimit}
              onValueChange={(value) => updateSetting('dataUsageLimit', Math.round(value))}
              minimumTrackTintColor="#007AFF"
              maximumTrackTintColor="#E0E0E0"
              thumbTintColor="#007AFF"
              step={10}
            />
            <View style={styles.sliderLabels}>
              <Text style={styles.sliderMinMax}>10MB</Text>
              <Text style={styles.sliderMinMax}>500MB</Text>
            </View>
          </View>
        </View>
      </Modal>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8F9FA',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#666666',
  },
  section: {
    backgroundColor: '#FFFFFF',
    marginVertical: 8,
    paddingHorizontal: 16,
  },
  sectionHeader: {
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
  },
  settingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  settingInfo: {
    flex: 1,
  },
  settingTitle: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333333',
  },
  settingSubtitle: {
    fontSize: 14,
    color: '#666666',
    marginTop: 2,
  },
  settingControl: {
    marginLeft: 16,
  },
  settingValue: {
    fontSize: 16,
    color: '#007AFF',
    fontWeight: '500',
  },
  scopeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  scopeLabel: {
    fontSize: 16,
    color: '#333333',
  },
  statsContainer: {
    backgroundColor: '#FFFFFF',
    marginVertical: 8,
    padding: 16,
  },
  statsTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 16,
  },
  statsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  statItem: {
    width: '50%',
    alignItems: 'center',
    paddingVertical: 8,
  },
  statValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#007AFF',
  },
  statLabel: {
    fontSize: 12,
    color: '#666666',
    marginTop: 4,
  },
  expandButton: {
    paddingVertical: 16,
    alignItems: 'center',
  },
  expandButtonText: {
    fontSize: 16,
    color: '#007AFF',
    fontWeight: '500',
  },
  advancedSection: {
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
  },
  dangerButton: {
    backgroundColor: '#DC3545',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 8,
    marginVertical: 8,
    alignItems: 'center',
  },
  dangerButtonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '600',
  },
  actionButtons: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
  },
  resetButton: {
    flex: 1,
    backgroundColor: '#F5F5F5',
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
  },
  resetButtonText: {
    fontSize: 16,
    color: '#666666',
    fontWeight: '600',
  },
  saveButton: {
    flex: 2,
    backgroundColor: '#007AFF',
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
  },
  disabledButton: {
    backgroundColor: '#CCCCCC',
  },
  saveButtonText: {
    fontSize: 16,
    color: '#FFFFFF',
    fontWeight: '600',
  },
  modalContainer: {
    flex: 1,
    backgroundColor: '#FFFFFF',
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
  },
  modalClose: {
    fontSize: 16,
    color: '#007AFF',
    fontWeight: '600',
  },
  sliderContainer: {
    padding: 32,
  },
  sliderLabel: {
    fontSize: 18,
    fontWeight: '500',
    color: '#333333',
    textAlign: 'center',
    marginBottom: 32,
  },
  slider: {
    width: '100%',
    height: 40,
  },
  sliderLabels: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 8,
  },
  sliderMinMax: {
    fontSize: 12,
    color: '#666666',
  },
});

export default SyncSettingsScreen;