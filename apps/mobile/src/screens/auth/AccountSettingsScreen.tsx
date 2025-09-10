import React, { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Alert,
  Switch,
  Share,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { useAuthStore } from '../../store/auth_store';

const AccountSettingsScreen: React.FC = () => {
  const navigation = useNavigation();
  const { user, logout } = useAuthStore();
  
  const [settings, setSettings] = useState({
    biometricAuth: true,
    pushNotifications: true,
    emailNotifications: false,
    marketingEmails: false,
    shareUsageData: false,
  });

  const handleToggleSetting = (key: keyof typeof settings) => {
    setSettings(prev => ({
      ...prev,
      [key]: !prev[key],
    }));
  };

  const handleExportData = async () => {
    try {
      // TODO: Implement data export API call
      const exportData = {
        profile: user,
        recipes: [], // Placeholder
        mealPlans: [], // Placeholder
        preferences: settings,
      };

      const dataString = JSON.stringify(exportData, null, 2);
      
      await Share.share({
        message: 'Your imkitchen data export',
        title: 'Data Export',
        url: `data:text/json;base64,${Buffer.from(dataString).toString('base64')}`,
      });
    } catch (error) {
      Alert.alert('Error', 'Failed to export data');
    }
  };

  const handleDeleteAccount = () => {
    Alert.alert(
      'Delete Account',
      'This will permanently delete your account and all associated data. This action cannot be undone.\n\nWould you like to export your data first?',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Export & Delete',
          style: 'destructive',
          onPress: () => {
            handleExportData();
            setTimeout(() => confirmAccountDeletion(), 1000);
          },
        },
        {
          text: 'Delete Without Export',
          style: 'destructive',
          onPress: confirmAccountDeletion,
        },
      ]
    );
  };

  const confirmAccountDeletion = () => {
    Alert.alert(
      'Final Confirmation',
      'Type "DELETE" to confirm account deletion:',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Confirm',
          style: 'destructive',
          onPress: async () => {
            try {
              // TODO: Implement account deletion API call
              Alert.alert('Account Deleted', 'Your account has been permanently deleted.');
              await logout();
            } catch (error) {
              Alert.alert('Error', 'Failed to delete account');
            }
          },
        },
      ]
    );
  };

  const handleChangePassword = () => {
    // TODO: Navigate to change password screen or implement in-app flow
    Alert.alert('Change Password', 'This feature will be implemented in a future update.');
  };

  const handleContactSupport = () => {
    Alert.alert(
      'Contact Support',
      'How would you like to contact support?',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Email', onPress: () => Alert.alert('Email', 'Opening email app...') },
        { text: 'In-App Chat', onPress: () => Alert.alert('Chat', 'Opening chat...') },
      ]
    );
  };

  const SettingItem: React.FC<{
    title: string;
    subtitle?: string;
    onPress?: () => void;
    rightElement?: React.ReactNode;
    destructive?: boolean;
  }> = ({ title, subtitle, onPress, rightElement, destructive = false }) => (
    <TouchableOpacity
      style={styles.settingItem}
      onPress={onPress}
      disabled={!onPress && !rightElement}
    >
      <View style={styles.settingContent}>
        <Text style={[styles.settingTitle, destructive && styles.destructiveText]}>
          {title}
        </Text>
        {subtitle && (
          <Text style={styles.settingSubtitle}>{subtitle}</Text>
        )}
      </View>
      {rightElement && <View style={styles.settingRight}>{rightElement}</View>}
      {onPress && !rightElement && (
        <Text style={styles.settingArrow}>›</Text>
      )}
    </TouchableOpacity>
  );

  return (
    <SafeAreaView style={styles.container}>
      <ScrollView contentContainerStyle={styles.scrollContent}>
        <View style={styles.header}>
          <TouchableOpacity
            style={styles.backButton}
            onPress={() => navigation.goBack()}
          >
            <Text style={styles.backButtonText}>← Back</Text>
          </TouchableOpacity>
          <Text style={styles.title}>Account Settings</Text>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Security</Text>
          
          <SettingItem
            title="Biometric Authentication"
            subtitle="Use Face ID or fingerprint to sign in"
            rightElement={
              <Switch
                value={settings.biometricAuth}
                onValueChange={() => handleToggleSetting('biometricAuth')}
                trackColor={{ false: '#d1d5db', true: '#fed7aa' }}
                thumbColor={settings.biometricAuth ? '#f97316' : '#f3f4f6'}
              />
            }
          />

          <SettingItem
            title="Change Password"
            subtitle="Update your account password"
            onPress={handleChangePassword}
          />
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Notifications</Text>
          
          <SettingItem
            title="Push Notifications"
            subtitle="Receive notifications on your device"
            rightElement={
              <Switch
                value={settings.pushNotifications}
                onValueChange={() => handleToggleSetting('pushNotifications')}
                trackColor={{ false: '#d1d5db', true: '#fed7aa' }}
                thumbColor={settings.pushNotifications ? '#f97316' : '#f3f4f6'}
              />
            }
          />

          <SettingItem
            title="Email Notifications"
            subtitle="Receive updates via email"
            rightElement={
              <Switch
                value={settings.emailNotifications}
                onValueChange={() => handleToggleSetting('emailNotifications')}
                trackColor={{ false: '#d1d5db', true: '#fed7aa' }}
                thumbColor={settings.emailNotifications ? '#f97316' : '#f3f4f6'}
              />
            }
          />

          <SettingItem
            title="Marketing Emails"
            subtitle="Receive promotional content"
            rightElement={
              <Switch
                value={settings.marketingEmails}
                onValueChange={() => handleToggleSetting('marketingEmails')}
                trackColor={{ false: '#d1d5db', true: '#fed7aa' }}
                thumbColor={settings.marketingEmails ? '#f97316' : '#f3f4f6'}
              />
            }
          />
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Privacy</Text>
          
          <SettingItem
            title="Share Usage Data"
            subtitle="Help improve imkitchen by sharing anonymous usage data"
            rightElement={
              <Switch
                value={settings.shareUsageData}
                onValueChange={() => handleToggleSetting('shareUsageData')}
                trackColor={{ false: '#d1d5db', true: '#fed7aa' }}
                thumbColor={settings.shareUsageData ? '#f97316' : '#f3f4f6'}
              />
            }
          />

          <SettingItem
            title="Export My Data"
            subtitle="Download a copy of your personal data"
            onPress={handleExportData}
          />
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Support</Text>
          
          <SettingItem
            title="Contact Support"
            subtitle="Get help with your account"
            onPress={handleContactSupport}
          />

          <SettingItem
            title="Privacy Policy"
            subtitle="View our privacy policy"
            onPress={() => Alert.alert('Privacy Policy', 'Opening privacy policy...')}
          />

          <SettingItem
            title="Terms of Service"
            subtitle="View terms of service"
            onPress={() => Alert.alert('Terms of Service', 'Opening terms of service...')}
          />
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Account</Text>
          
          <SettingItem
            title="Sign Out"
            subtitle="Sign out of your account"
            onPress={async () => {
              Alert.alert(
                'Sign Out',
                'Are you sure you want to sign out?',
                [
                  { text: 'Cancel', style: 'cancel' },
                  {
                    text: 'Sign Out',
                    style: 'destructive',
                    onPress: logout,
                  },
                ]
              );
            }}
          />

          <SettingItem
            title="Delete Account"
            subtitle="Permanently delete your account and data"
            onPress={handleDeleteAccount}
            destructive
          />
        </View>

        <View style={styles.footer}>
          <Text style={styles.footerText}>
            imkitchen v1.0.0{'\n'}
            User ID: {user?.id?.slice(0, 8)}...
          </Text>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  scrollContent: {
    flexGrow: 1,
  },
  header: {
    padding: 24,
    paddingBottom: 16,
  },
  backButton: {
    alignSelf: 'flex-start',
    marginBottom: 16,
    padding: 8,
  },
  backButtonText: {
    fontSize: 16,
    color: '#f97316',
    fontWeight: '600',
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#1a1a1a',
  },
  section: {
    marginBottom: 32,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1a1a1a',
    marginBottom: 16,
    paddingHorizontal: 24,
  },
  settingItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 16,
    paddingHorizontal: 24,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  settingContent: {
    flex: 1,
  },
  settingTitle: {
    fontSize: 16,
    fontWeight: '500',
    color: '#1a1a1a',
    marginBottom: 2,
  },
  settingSubtitle: {
    fontSize: 14,
    color: '#6b7280',
  },
  settingRight: {
    marginLeft: 16,
  },
  settingArrow: {
    fontSize: 20,
    color: '#9ca3af',
    fontWeight: '300',
  },
  destructiveText: {
    color: '#dc2626',
  },
  footer: {
    padding: 24,
    paddingTop: 16,
    alignItems: 'center',
  },
  footerText: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'center',
    lineHeight: 18,
  },
});

export default AccountSettingsScreen;