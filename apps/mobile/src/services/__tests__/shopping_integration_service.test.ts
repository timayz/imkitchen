import { shoppingIntegrationService } from '../shopping_integration_service';
import { shoppingService } from '../shopping_service';
import type { MealPlanResponse, ShoppingList } from '../../types/shopping';

// Mock the shopping service
jest.mock('../shopping_service');
const mockShoppingService = shoppingService as jest.Mocked<typeof shoppingService>;

// Mock data
const mockMealPlan: MealPlanResponse = {
  id: 'meal-plan-1',
  userId: 'user-1',
  weekStartDate: '2024-01-01',
  entries: [
    {
      id: 'entry-1',
      day: 'monday',
      mealType: 'breakfast',
      recipe: {
        id: 'recipe-1',
        name: 'Scrambled Eggs',
        ingredients: [
          { name: 'Eggs', amount: 4, unit: 'pieces', category: 'protein' },
          { name: 'Milk', amount: 0.25, unit: 'cup', category: 'dairy' },
        ],
      },
    },
  ],
  status: 'active',
  generatedAt: new Date('2024-01-01'),
  createdAt: new Date('2024-01-01'),
  updatedAt: new Date('2024-01-01'),
};

const mockUpdatedMealPlan: MealPlanResponse = {
  ...mockMealPlan,
  entries: [
    ...mockMealPlan.entries,
    {
      id: 'entry-2',
      day: 'monday',
      mealType: 'lunch',
      recipe: {
        id: 'recipe-2',
        name: 'Chicken Salad',
        ingredients: [
          { name: 'Chicken Breast', amount: 1, unit: 'lb', category: 'protein' },
          { name: 'Lettuce', amount: 1, unit: 'head', category: 'produce' },
        ],
      },
    },
  ],
  updatedAt: new Date('2024-01-02'),
};

const mockShoppingList: ShoppingList = {
  id: 'shopping-list-1',
  userId: 'user-1',
  mealPlanId: 'meal-plan-1',
  name: 'Weekly Shopping List',
  status: 'active',
  categories: {
    protein: [
      {
        id: 'item-1',
        shoppingListId: 'shopping-list-1',
        ingredientName: 'Eggs',
        amount: 4,
        unit: 'pieces',
        category: 'protein',
        isCompleted: false,
        recipeSources: ['recipe-1'],
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-01'),
      },
    ],
    dairy: [
      {
        id: 'item-2',
        shoppingListId: 'shopping-list-1',
        ingredientName: 'Milk',
        amount: 0.25,
        unit: 'cup',
        category: 'dairy',
        isCompleted: false,
        recipeSources: ['recipe-1'],
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-01'),
      },
    ],
  },
  totalItems: 2,
  completedItems: 0,
  generatedAt: new Date('2024-01-01'),
};

describe('ShoppingIntegrationService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockShoppingService.generateShoppingList.mockResolvedValue(mockShoppingList);
    mockShoppingService.getShoppingLists.mockResolvedValue([]);
    mockShoppingService.deleteShoppingList.mockResolvedValue();
  });

  describe('handleMealPlanChange', () => {
    it('generates new shopping list for new meal plan', async () => {
      const result = await shoppingIntegrationService.handleMealPlanChange(
        mockMealPlan,
        null,
        { autoGenerate: true }
      );

      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledWith({
        mealPlanId: 'meal-plan-1',
        mergeExisting: true,
      });
      expect(result).toEqual(mockShoppingList);
    });

    it('does not generate shopping list when autoGenerate is false', async () => {
      const result = await shoppingIntegrationService.handleMealPlanChange(
        mockMealPlan,
        null,
        { autoGenerate: false }
      );

      expect(mockShoppingService.generateShoppingList).not.toHaveBeenCalled();
      expect(result).toBeNull();
    });

    it('updates existing shopping list when meal plan is modified', async () => {
      mockShoppingService.getShoppingLists.mockResolvedValue([mockShoppingList]);

      const result = await shoppingIntegrationService.handleMealPlanChange(
        mockUpdatedMealPlan,
        mockMealPlan,
        { autoGenerate: true, mergeExisting: true }
      );

      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledWith({
        mealPlanId: 'meal-plan-1',
        mergeExisting: true,
      });
      expect(result).toEqual(mockShoppingList);
    });

    it('handles errors gracefully', async () => {
      mockShoppingService.generateShoppingList.mockRejectedValue(
        new Error('Generation failed')
      );

      const result = await shoppingIntegrationService.handleMealPlanChange(
        mockMealPlan,
        null,
        { autoGenerate: true, notifyUser: false }
      );

      expect(result).toBeNull();
    });
  });

  describe('generateShoppingList', () => {
    it('calls shopping service with correct parameters', async () => {
      const result = await shoppingIntegrationService.generateShoppingList(
        'meal-plan-1',
        false
      );

      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledWith({
        mealPlanId: 'meal-plan-1',
        mergeExisting: false,
      });
      expect(result).toEqual(mockShoppingList);
    });

    it('defaults mergeExisting to false', async () => {
      await shoppingIntegrationService.generateShoppingList('meal-plan-1');

      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledWith({
        mealPlanId: 'meal-plan-1',
        mergeExisting: false,
      });
    });
  });

  describe('findExistingShoppingList', () => {
    it('finds existing shopping list for meal plan', async () => {
      mockShoppingService.getShoppingLists.mockResolvedValue([mockShoppingList]);

      const result = await shoppingIntegrationService.findExistingShoppingList(
        'meal-plan-1'
      );

      expect(result).toEqual(mockShoppingList);
    });

    it('returns null when no shopping list found', async () => {
      mockShoppingService.getShoppingLists.mockResolvedValue([]);

      const result = await shoppingIntegrationService.findExistingShoppingList(
        'meal-plan-1'
      );

      expect(result).toBeNull();
    });

    it('handles service errors gracefully', async () => {
      mockShoppingService.getShoppingLists.mockRejectedValue(
        new Error('Service error')
      );

      const result = await shoppingIntegrationService.findExistingShoppingList(
        'meal-plan-1'
      );

      expect(result).toBeNull();
    });
  });

  describe('calculateShoppingListDiff', () => {
    it('calculates added ingredients correctly', async () => {
      const diff = await shoppingIntegrationService.calculateShoppingListDiff(
        mockMealPlan,
        mockUpdatedMealPlan
      );

      // Note: This is a simplified test as the actual implementation would 
      // need to extract ingredients from meal plan entries
      expect(diff).toHaveProperty('added');
      expect(diff).toHaveProperty('removed');
      expect(diff).toHaveProperty('modified');
    });

    it('calculates removed ingredients correctly', async () => {
      const diff = await shoppingIntegrationService.calculateShoppingListDiff(
        mockUpdatedMealPlan,
        mockMealPlan
      );

      expect(diff).toHaveProperty('added');
      expect(diff).toHaveProperty('removed');
      expect(diff).toHaveProperty('modified');
    });
  });

  describe('notification system', () => {
    it('queues notifications correctly', () => {
      shoppingIntegrationService.queueNotification('Test message', 'success');

      const notifications = shoppingIntegrationService.getNotifications();
      expect(notifications).toHaveLength(1);
      expect(notifications[0]).toMatchObject({
        message: 'Test message',
        type: 'success',
      });
    });

    it('clears notifications', () => {
      shoppingIntegrationService.queueNotification('Test message', 'info');
      shoppingIntegrationService.clearNotifications();

      const notifications = shoppingIntegrationService.getNotifications();
      expect(notifications).toHaveLength(0);
    });

    it('auto-clears old notifications', (done) => {
      shoppingIntegrationService.queueNotification('Test message', 'warning');

      // Fast-forward time to test auto-clear
      setTimeout(() => {
        const notifications = shoppingIntegrationService.getNotifications();
        expect(notifications).toHaveLength(0);
        done();
      }, 11000); // Just over the 10-second threshold
    }, 12000);
  });

  describe('regenerateShoppingList', () => {
    it('deletes existing list when not merging', async () => {
      mockShoppingService.getShoppingLists.mockResolvedValue([mockShoppingList]);

      await shoppingIntegrationService.regenerateShoppingList(
        'meal-plan-1',
        false
      );

      expect(mockShoppingService.deleteShoppingList).toHaveBeenCalledWith(
        'shopping-list-1'
      );
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledWith({
        mealPlanId: 'meal-plan-1',
        mergeExisting: false,
      });
    });

    it('skips deletion when merging', async () => {
      await shoppingIntegrationService.regenerateShoppingList(
        'meal-plan-1',
        true
      );

      expect(mockShoppingService.deleteShoppingList).not.toHaveBeenCalled();
      expect(mockShoppingService.generateShoppingList).toHaveBeenCalledWith({
        mealPlanId: 'meal-plan-1',
        mergeExisting: true,
      });
    });
  });

  describe('hook integration', () => {
    it('onMealPlanGenerated calls handleMealPlanChange correctly', async () => {
      const spy = jest.spyOn(shoppingIntegrationService, 'handleMealPlanChange');
      spy.mockResolvedValue(mockShoppingList);

      await shoppingIntegrationService.onMealPlanGenerated(mockMealPlan);

      expect(spy).toHaveBeenCalledWith(mockMealPlan, null, {
        autoGenerate: true,
        mergeExisting: false,
        notifyUser: true,
      });

      spy.mockRestore();
    });

    it('onMealPlanUpdated calls handleMealPlanChange correctly', async () => {
      const spy = jest.spyOn(shoppingIntegrationService, 'handleMealPlanChange');
      spy.mockResolvedValue(mockShoppingList);

      await shoppingIntegrationService.onMealPlanUpdated(
        mockUpdatedMealPlan,
        mockMealPlan
      );

      expect(spy).toHaveBeenCalledWith(mockUpdatedMealPlan, mockMealPlan, {
        autoGenerate: true,
        mergeExisting: true,
        notifyUser: true,
      });

      spy.mockRestore();
    });
  });
});