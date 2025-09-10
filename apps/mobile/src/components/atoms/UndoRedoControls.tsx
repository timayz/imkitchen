import React from 'react';
import {
  View,
  TouchableOpacity,
  Text,
  StyleSheet,
  AccessibilityInfo,
} from 'react-native';

interface UndoRedoControlsProps {
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  undoDescription?: string;
  redoDescription?: string;
  size?: 'small' | 'medium' | 'large';
  style?: 'minimal' | 'outlined' | 'filled';
  orientation?: 'horizontal' | 'vertical';
  showLabels?: boolean;
  disabled?: boolean;
}

export const UndoRedoControls: React.FC<UndoRedoControlsProps> = ({
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  undoDescription,
  redoDescription,
  size = 'medium',
  style = 'outlined',
  orientation = 'horizontal',
  showLabels = false,
  disabled = false,
}) => {
  const handleUndo = () => {
    if (!disabled && canUndo) {
      onUndo();
      
      // Announce to screen readers
      AccessibilityInfo.announceForAccessibility(
        undoDescription ? `Undid: ${undoDescription}` : 'Undid last action'
      );
    }
  };

  const handleRedo = () => {
    if (!disabled && canRedo) {
      onRedo();
      
      // Announce to screen readers
      AccessibilityInfo.announceForAccessibility(
        redoDescription ? `Redid: ${redoDescription}` : 'Redid last undone action'
      );
    }
  };

  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return {
          buttonSize: 32,
          iconSize: 16,
          fontSize: 10,
          spacing: 4,
        };
      case 'large':
        return {
          buttonSize: 48,
          iconSize: 24,
          fontSize: 16,
          spacing: 8,
        };
      default: // medium
        return {
          buttonSize: 40,
          iconSize: 20,
          fontSize: 14,
          spacing: 6,
        };
    }
  };

  const getStyleVariant = () => {
    switch (style) {
      case 'minimal':
        return {
          button: styles.minimalButton,
          buttonDisabled: styles.minimalButtonDisabled,
          text: styles.minimalText,
          textDisabled: styles.minimalTextDisabled,
        };
      case 'filled':
        return {
          button: styles.filledButton,
          buttonDisabled: styles.filledButtonDisabled,
          text: styles.filledText,
          textDisabled: styles.filledTextDisabled,
        };
      default: // outlined
        return {
          button: styles.outlinedButton,
          buttonDisabled: styles.outlinedButtonDisabled,
          text: styles.outlinedText,
          textDisabled: styles.outlinedTextDisabled,
        };
    }
  };

  const sizeStyles = getSizeStyles();
  const styleVariant = getStyleVariant();

  const containerStyle = [
    styles.container,
    orientation === 'vertical' ? styles.verticalContainer : styles.horizontalContainer,
    { gap: sizeStyles.spacing },
  ];

  const undoButtonStyle = [
    styles.button,
    styleVariant.button,
    {
      width: sizeStyles.buttonSize,
      height: sizeStyles.buttonSize,
      borderRadius: sizeStyles.buttonSize / 2,
    },
    (!canUndo || disabled) && [styles.buttonDisabled, styleVariant.buttonDisabled],
  ];

  const redoButtonStyle = [
    styles.button,
    styleVariant.button,
    {
      width: sizeStyles.buttonSize,
      height: sizeStyles.buttonSize,
      borderRadius: sizeStyles.buttonSize / 2,
    },
    (!canRedo || disabled) && [styles.buttonDisabled, styleVariant.buttonDisabled],
  ];

  const undoTextStyle = [
    styleVariant.text,
    { fontSize: sizeStyles.iconSize },
    (!canUndo || disabled) && styleVariant.textDisabled,
  ];

  const redoTextStyle = [
    styleVariant.text,
    { fontSize: sizeStyles.iconSize },
    (!canRedo || disabled) && styleVariant.textDisabled,
  ];

  return (
    <View style={containerStyle}>
      {/* Undo Button */}
      <View style={showLabels ? styles.buttonWithLabel : undefined}>
        <TouchableOpacity
          style={undoButtonStyle}
          onPress={handleUndo}
          disabled={!canUndo || disabled}
          accessibilityRole="button"
          accessibilityLabel="Undo"
          accessibilityHint={
            canUndo && undoDescription 
              ? `Undo: ${undoDescription}` 
              : canUndo 
                ? "Undo last action" 
                : "No actions to undo"
          }
          accessibilityState={{
            disabled: !canUndo || disabled,
          }}
        >
          <Text style={undoTextStyle}>↶</Text>
        </TouchableOpacity>
        {showLabels && (
          <Text style={[styles.label, { fontSize: sizeStyles.fontSize }]}>
            Undo
          </Text>
        )}
      </View>

      {/* Redo Button */}
      <View style={showLabels ? styles.buttonWithLabel : undefined}>
        <TouchableOpacity
          style={redoButtonStyle}
          onPress={handleRedo}
          disabled={!canRedo || disabled}
          accessibilityRole="button"
          accessibilityLabel="Redo"
          accessibilityHint={
            canRedo && redoDescription 
              ? `Redo: ${redoDescription}` 
              : canRedo 
                ? "Redo last undone action" 
                : "No actions to redo"
          }
          accessibilityState={{
            disabled: !canRedo || disabled,
          }}
        >
          <Text style={redoTextStyle}>↷</Text>
        </TouchableOpacity>
        {showLabels && (
          <Text style={[styles.label, { fontSize: sizeStyles.fontSize }]}>
            Redo
          </Text>
        )}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
  },
  horizontalContainer: {
    flexDirection: 'row',
  },
  verticalContainer: {
    flexDirection: 'column',
  },
  button: {
    justifyContent: 'center',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.2,
    shadowRadius: 2,
    elevation: 2,
  },
  buttonDisabled: {
    shadowOpacity: 0.1,
    elevation: 1,
  },
  buttonWithLabel: {
    alignItems: 'center',
    gap: 4,
  },

  // Minimal style variant
  minimalButton: {
    backgroundColor: 'transparent',
    shadowOpacity: 0,
    elevation: 0,
  },
  minimalButtonDisabled: {
    backgroundColor: 'transparent',
  },
  minimalText: {
    color: '#007AFF',
    fontWeight: '600',
  },
  minimalTextDisabled: {
    color: '#C7C7CC',
  },

  // Outlined style variant
  outlinedButton: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#007AFF',
  },
  outlinedButtonDisabled: {
    borderColor: '#C7C7CC',
    backgroundColor: '#F2F2F7',
  },
  outlinedText: {
    color: '#007AFF',
    fontWeight: '600',
  },
  outlinedTextDisabled: {
    color: '#C7C7CC',
  },

  // Filled style variant
  filledButton: {
    backgroundColor: '#007AFF',
  },
  filledButtonDisabled: {
    backgroundColor: '#C7C7CC',
  },
  filledText: {
    color: '#FFFFFF',
    fontWeight: '600',
  },
  filledTextDisabled: {
    color: '#FFFFFF',
    opacity: 0.7,
  },

  label: {
    color: '#666666',
    textAlign: 'center',
    marginTop: 2,
  },
});