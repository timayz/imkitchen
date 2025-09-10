import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Platform,
} from 'react-native';

interface SocialLoginButtonsProps {
  disabled?: boolean;
}

const SocialLoginButtons: React.FC<SocialLoginButtonsProps> = ({ disabled = false }) => {
  const handleGoogleLogin = async () => {
    try {
      // TODO: Implement Google Sign-In
      Alert.alert('Google Sign-In', 'Google authentication will be implemented in a future update.');
    } catch (error) {
      Alert.alert('Error', 'Google sign-in failed');
    }
  };

  const handleAppleLogin = async () => {
    try {
      // TODO: Implement Apple Sign-In
      Alert.alert('Apple Sign-In', 'Apple authentication will be implemented in a future update.');
    } catch (error) {
      Alert.alert('Error', 'Apple sign-in failed');
    }
  };

  return (
    <View style={styles.container}>
      <TouchableOpacity
        style={[styles.socialButton, styles.googleButton, disabled && styles.disabledButton]}
        onPress={handleGoogleLogin}
        disabled={disabled}
      >
        <View style={styles.buttonContent}>
          <Text style={styles.googleIcon}>G</Text>
          <Text style={styles.googleText}>Continue with Google</Text>
        </View>
      </TouchableOpacity>

      {Platform.OS === 'ios' && (
        <TouchableOpacity
          style={[styles.socialButton, styles.appleButton, disabled && styles.disabledButton]}
          onPress={handleAppleLogin}
          disabled={disabled}
        >
          <View style={styles.buttonContent}>
            <Text style={styles.appleIcon}></Text>
            <Text style={styles.appleText}>Continue with Apple</Text>
          </View>
        </TouchableOpacity>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    gap: 12,
  },
  socialButton: {
    height: 48,
    borderRadius: 8,
    borderWidth: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  buttonContent: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  googleButton: {
    backgroundColor: '#fff',
    borderColor: '#d1d5db',
  },
  googleIcon: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#4285f4',
    backgroundColor: '#4285f4',
    color: '#fff',
    width: 24,
    height: 24,
    borderRadius: 12,
    textAlign: 'center',
    lineHeight: 24,
  },
  googleText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#374151',
  },
  appleButton: {
    backgroundColor: '#000',
    borderColor: '#000',
  },
  appleIcon: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    width: 24,
    height: 24,
    textAlign: 'center',
    lineHeight: 24,
  },
  appleText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#fff',
  },
  disabledButton: {
    opacity: 0.6,
  },
});

export default SocialLoginButtons;