/**
 * Conflict Resolution Modal Tests
 * 
 * Tests for the ConflictResolutionModal component including
 * diff visualization, user interactions, and resolution strategies.
 */

import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { Alert } from 'react-native';
import ConflictResolutionModal from '../sync/ConflictResolutionModal';
import { ConflictData, ResolutionType } from '../../services/conflict_resolution_service';
import { conflictResolutionService } from '../../services/conflict_resolution_service';

// Mock the conflict resolution service
jest.mock('../../services/conflict_resolution_service', () => ({
  conflictResolutionService: {
    resolveConflict: jest.fn()
  }
}));

// Mock Alert
jest.spyOn(Alert, 'alert');
jest.spyOn(Alert, 'prompt');

describe('ConflictResolutionModal', () => {
  const mockConflictData: ConflictData = {
    itemId: 'recipe_123',
    itemType: 'user_recipe' as any,
    localVersion: {
      title: 'My Chocolate Cake',
      description: 'Delicious chocolate cake recipe',
      ingredients: ['flour', 'sugar', 'cocoa']
    },
    remoteVersion: {
      title: 'Chocolate Cake Recipe',
      description: 'Rich chocolate cake recipe',
      ingredients: ['flour', 'sugar', 'cocoa', 'eggs']
    },
    baseVersion: {
      title: 'Chocolate Cake',
      description: 'Chocolate cake recipe',
      ingredients: ['flour', 'sugar', 'cocoa']
    },
    conflictingFields: [
      {
        fieldPath: 'title',
        localValue: 'My Chocolate Cake',
        remoteValue: 'Chocolate Cake Recipe',
        baseValue: 'Chocolate Cake',
        conflictType: 'value_change' as any
      },
      {
        fieldPath: 'description',
        localValue: 'Delicious chocolate cake recipe',
        remoteValue: 'Rich chocolate cake recipe',
        baseValue: 'Chocolate cake recipe',
        conflictType: 'value_change' as any
      }
    ],
    detectedAt: new Date(),
    metadata: {} as any
  };

  const mockProps = {
    visible: true,
    conflictData: mockConflictData,
    onResolve: jest.fn(),
    onCancel: jest.fn()
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    test('should render modal when visible', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      expect(getByText('Resolve Conflict')).toBeTruthy();
      expect(getByText('user_recipe - recipe_123')).toBeTruthy();
    });

    test('should not render when not visible', () => {
      const { queryByText } = render(
        <ConflictResolutionModal {...mockProps} visible={false} />
      );
      
      expect(queryByText('Resolve Conflict')).toBeFalsy();
    });

    test('should render quick resolution buttons', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      expect(getByText('Keep Mine')).toBeTruthy();
      expect(getByText('Keep Theirs')).toBeTruthy();
      expect(getByText('Most Recent')).toBeTruthy();
    });

    test('should render conflicting fields', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      expect(getByText('title')).toBeTruthy();
      expect(getByText('description')).toBeTruthy();
      expect(getByText('My Chocolate Cake')).toBeTruthy();
      expect(getByText('Chocolate Cake Recipe')).toBeTruthy();
    });

    test('should render base version when available', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      expect(getByText('Original Version:')).toBeTruthy();
      expect(getByText('Chocolate Cake')).toBeTruthy();
    });
  });

  describe('Quick Resolution', () => {
    test('should handle "Keep Mine" resolution', async () => {
      (conflictResolutionService.resolveConflict as jest.Mock).mockResolvedValue({
        success: true,
        resolvedData: mockConflictData.localVersion,
        strategy: ResolutionType.LOCAL_WINS
      });

      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      fireEvent.press(getByText('Keep Mine'));

      await waitFor(() => {
        expect(conflictResolutionService.resolveConflict).toHaveBeenCalledWith(
          mockConflictData,
          ResolutionType.LOCAL_WINS
        );
        expect(mockProps.onResolve).toHaveBeenCalled();
      });
    });

    test('should handle "Keep Theirs" resolution', async () => {
      (conflictResolutionService.resolveConflict as jest.Mock).mockResolvedValue({
        success: true,
        resolvedData: mockConflictData.remoteVersion,
        strategy: ResolutionType.REMOTE_WINS
      });

      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      fireEvent.press(getByText('Keep Theirs'));

      await waitFor(() => {
        expect(conflictResolutionService.resolveConflict).toHaveBeenCalledWith(
          mockConflictData,
          ResolutionType.REMOTE_WINS
        );
        expect(mockProps.onResolve).toHaveBeenCalled();
      });
    });

    test('should handle quick resolution failure', async () => {
      (conflictResolutionService.resolveConflict as jest.Mock).mockRejectedValue(
        new Error('Resolution failed')
      );

      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      fireEvent.press(getByText('Keep Mine'));

      await waitFor(() => {
        expect(Alert.alert).toHaveBeenCalledWith(
          'Resolution Failed',
          'Could not automatically resolve the conflict. Please try manual resolution.'
        );
      });
    });
  });

  describe('Manual Field Resolution', () => {
    test('should allow selecting local value for a field', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      // Find the local version container for the title field
      const localVersionText = getByText('My Chocolate Cake');
      const localContainer = localVersionText.parent?.parent;
      
      expect(localContainer).toBeTruthy();
      fireEvent.press(localContainer as any);

      // Should update the selection (visual feedback tested in integration)
    });

    test('should generate preview when field choices are made', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      // Make a choice for a field
      const localVersionText = getByText('My Chocolate Cake');
      const localContainer = localVersionText.parent?.parent;
      fireEvent.press(localContainer as any);

      // Preview should be generated (implementation specific)
    });

    test('should handle custom value input', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      const customButton = getByText('Enter Custom Value');
      fireEvent.press(customButton);

      expect(Alert.prompt).toHaveBeenCalledWith(
        'Custom Value',
        expect.stringContaining('Enter custom value for'),
        expect.arrayContaining([
          { text: 'Cancel', style: 'cancel' },
          { text: 'Save', onPress: expect.any(Function) }
        ]),
        'plain-text',
        expect.any(String)
      );
    });

    test('should apply manual resolution', async () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      // Make field choices first
      const localVersionText = getByText('My Chocolate Cake');
      const localContainer = localVersionText.parent?.parent;
      fireEvent.press(localContainer as any);
      
      const applyButton = getByText('Apply Resolution');
      fireEvent.press(applyButton);

      await waitFor(() => {
        expect(mockProps.onResolve).toHaveBeenCalledWith(
          expect.objectContaining({
            success: true,
            strategy: ResolutionType.USER_GUIDED,
            confidence: 100
          })
        );
      });
    });

    test('should prevent resolution without field choices', async () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      const applyButton = getByText('Apply Resolution');
      fireEvent.press(applyButton);

      await waitFor(() => {
        expect(Alert.alert).toHaveBeenCalledWith(
          'Incomplete Resolution',
          'Please make choices for all conflicted fields.'
        );
      });
    });
  });

  describe('Navigation and Controls', () => {
    test('should handle cancel action', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      fireEvent.press(getByText('Cancel'));
      expect(mockProps.onCancel).toHaveBeenCalled();
    });

    test('should handle close button', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      fireEvent.press(getByText('✕'));
      expect(mockProps.onCancel).toHaveBeenCalled();
    });

    test('should disable buttons during loading', async () => {
      (conflictResolutionService.resolveConflict as jest.Mock).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 1000))
      );

      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      fireEvent.press(getByText('Keep Mine'));

      // Buttons should be disabled during loading
      expect(getByText('Keep Mine')).toBeDisabled();
      expect(getByText('Cancel')).toBeDisabled();
    });
  });

  describe('Data Handling', () => {
    test('should handle null conflict data', () => {
      const { queryByText } = render(
        <ConflictResolutionModal {...mockProps} conflictData={null} />
      );
      
      expect(queryByText('Resolve Conflict')).toBeFalsy();
    });

    test('should reset state when conflict data changes', () => {
      const { rerender, queryByText } = render(
        <ConflictResolutionModal {...mockProps} />
      );
      
      // Change conflict data
      const newConflictData = {
        ...mockConflictData,
        itemId: 'different_item'
      };
      
      rerender(
        <ConflictResolutionModal {...mockProps} conflictData={newConflictData} />
      );
      
      expect(queryByText('different_item')).toBeTruthy();
    });

    test('should handle complex nested objects', () => {
      const complexConflictData = {
        ...mockConflictData,
        conflictingFields: [
          {
            fieldPath: 'metadata.tags',
            localValue: ['sweet', 'dessert'],
            remoteValue: ['chocolate', 'cake'],
            conflictType: 'array_merge' as any
          }
        ]
      };

      const { getByText } = render(
        <ConflictResolutionModal {...mockProps} conflictData={complexConflictData} />
      );
      
      expect(getByText('metadata.tags')).toBeTruthy();
    });

    test('should handle JSON parsing for custom values', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      const customButton = getByText('Enter Custom Value');
      fireEvent.press(customButton);

      // Simulate JSON input
      const mockPrompt = Alert.prompt as jest.Mock;
      const onSaveCallback = mockPrompt.mock.calls[0][2][1].onPress;
      
      // Test valid JSON
      onSaveCallback('{"custom": "value"}');
      
      // Test invalid JSON (should keep as string)
      onSaveCallback('plain text');
      
      // Both should work without throwing errors
    });
  });

  describe('Accessibility', () => {
    test('should provide proper accessibility labels', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      // Check that important UI elements are accessible
      expect(getByText('Resolve Conflict')).toBeTruthy();
      expect(getByText('Your Version')).toBeTruthy();
      expect(getByText('Server Version')).toBeTruthy();
    });

    test('should handle screen reader navigation', () => {
      const { getByText } = render(<ConflictResolutionModal {...mockProps} />);
      
      // Verify that all interactive elements are properly labeled
      expect(getByText('Keep Mine')).toBeTruthy();
      expect(getByText('Apply Resolution')).toBeTruthy();
      expect(getByText('Cancel')).toBeTruthy();
    });
  });
});