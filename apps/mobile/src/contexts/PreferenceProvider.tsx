import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { usePreferenceStore } from '../store/preference_store';
import { usePreferences } from '../hooks/usePreferences';

interface PreferenceContextValue {
  isInitialized: boolean;
  hasLoadedPreferences: boolean;
}

const PreferenceContext = createContext<PreferenceContextValue>({
  isInitialized: false,
  hasLoadedPreferences: false,
});

interface PreferenceProviderProps {
  children: ReactNode;
}

/**
 * Provider component that initializes preference state for the entire app
 * and provides context about the initialization status
 */
export const PreferenceProvider: React.FC<PreferenceProviderProps> = ({ children }) => {
  const { refreshPreferences, hasPreferences, isLoading, error } = usePreferences();
  const lastUpdated = usePreferenceStore(state => state.lastUpdated);

  // Initialize preferences when the provider mounts
  useEffect(() => {
    const initializePreferences = async () => {
      try {
        // Only load if we haven't loaded preferences yet
        if (!hasPreferences && !isLoading) {
          await refreshPreferences();
        }
      } catch (err) {
        console.error('Failed to initialize preferences:', err);
        // Don't throw - let the app continue with default preferences
      }
    };

    initializePreferences();
  }, [hasPreferences, isLoading, refreshPreferences]);

  const contextValue: PreferenceContextValue = {
    isInitialized: true,
    hasLoadedPreferences: !!lastUpdated || !!error, // Consider initialized even with error
  };

  return (
    <PreferenceContext.Provider value={contextValue}>
      {children}
    </PreferenceContext.Provider>
  );
};

/**
 * Hook to access preference provider context
 */
export const usePreferenceProvider = () => {
  const context = useContext(PreferenceContext);
  if (!context) {
    throw new Error('usePreferenceProvider must be used within a PreferenceProvider');
  }
  return context;
};

/**
 * HOC for components that require preferences to be loaded
 */
export const withPreferences = <P extends object>(
  Component: React.ComponentType<P>
): React.FC<P> => {
  const WrappedComponent: React.FC<P> = (props) => {
    const { hasLoadedPreferences } = usePreferenceProvider();
    const { isLoading, error } = usePreferences();

    // Show loading state while preferences are being loaded
    if (!hasLoadedPreferences && isLoading) {
      return (
        <div style={{ 
          flex: 1, 
          justifyContent: 'center', 
          alignItems: 'center', 
          padding: 32 
        }}>
          <div>Loading preferences...</div>
        </div>
      );
    }

    // Show error state if preferences failed to load
    if (!hasLoadedPreferences && error) {
      return (
        <div style={{ 
          flex: 1, 
          justifyContent: 'center', 
          alignItems: 'center', 
          padding: 32 
        }}>
          <div style={{ color: '#F44336', textAlign: 'center' }}>
            Failed to load preferences: {error}
          </div>
        </div>
      );
    }

    return <Component {...props} />;
  };

  WrappedComponent.displayName = `withPreferences(${Component.displayName || Component.name})`;
  
  return WrappedComponent;
};