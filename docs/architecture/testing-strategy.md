# Testing Strategy

## Testing Pyramid

```text
        E2E Tests (Few)
       /              \
    Integration Tests (Some)
   /                      \
Frontend Unit Tests    Backend Unit Tests
    (Many)                 (Many)
```

## Test Organization

### Frontend Tests

```text
tests/
├── components/
│   ├── ui/
│   │   ├── button.test.tsx
│   │   └── input.test.tsx
│   ├── inventory/
│   │   ├── inventory-list.test.tsx
│   │   └── inventory-item.test.tsx
│   └── cooking/
│       ├── cooking-interface.test.tsx
│       └── timer-manager.test.tsx
├── hooks/
│   ├── use-auth.test.ts
│   ├── use-inventory.test.ts
│   └── use-voice.test.ts
├── services/
│   ├── inventory-service.test.ts
│   └── recipe-service.test.ts
└── utils/
    ├── api-client.test.ts
    └── validators.test.ts
```

### Backend Tests

```text
tests/
├── api/
│   ├── auth/
│   │   ├── login.test.ts
│   │   └── register.test.ts
│   ├── inventory/
│   │   ├── get-items.test.ts
│   │   └── create-item.test.ts
│   └── recipes/
│       ├── search.test.ts
│       └── suggestions.test.ts
├── services/
│   ├── inventory-service.test.ts
│   ├── recipe-service.test.ts
│   └── voice-service.test.ts
├── repositories/
│   ├── inventory-repository.test.ts
│   └── recipe-repository.test.ts
└── middleware/
    ├── auth-middleware.test.ts
    └── error-handler.test.ts
```

### E2E Tests

```text
tests/e2e/
├── auth/
│   ├── login.spec.ts
│   └── registration.spec.ts
├── inventory/
│   ├── add-items.spec.ts
│   ├── expiration-alerts.spec.ts
│   └── item-management.spec.ts
├── recipes/
│   ├── search-recipes.spec.ts
│   ├── save-favorites.spec.ts
│   └── ingredient-suggestions.spec.ts
├── meal-planning/
│   ├── create-meal-plan.spec.ts
│   ├── drag-drop-recipes.spec.ts
│   └── family-coordination.spec.ts
├── shopping/
│   ├── generate-list.spec.ts
│   ├── mark-purchased.spec.ts
│   └── store-organization.spec.ts
├── cooking/
│   ├── cooking-mode.spec.ts
│   ├── timer-management.spec.ts
│   └── voice-commands.spec.ts
└── voice/
    ├── voice-navigation.spec.ts
    └── hands-free-cooking.spec.ts
```

## Test Examples

### Frontend Component Test

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { InventoryList } from '@/components/inventory/inventory-list';
import { InventoryService } from '@/lib/services/inventory-service';

// Mock the service
jest.mock('@/lib/services/inventory-service');
const mockInventoryService = InventoryService as jest.Mocked<typeof InventoryService>;

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('InventoryList', () => {
  const mockItems = [
    {
      id: '1',
      name: 'Tomatoes',
      quantity: 3,
      unit: 'pieces',
      category: 'vegetables',
      location: 'refrigerator',
      expirationDate: new Date('2025-09-20'),
    },
    {
      id: '2',
      name: 'Milk',
      quantity: 1,
      unit: 'liters',
      category: 'dairy',
      location: 'refrigerator',
      expirationDate: new Date('2025-09-16'), // Expiring soon
    },
  ];

  beforeEach(() => {
    mockInventoryService.getItems.mockResolvedValue(mockItems);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  it('renders inventory items correctly', async () => {
    render(<InventoryList />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Tomatoes')).toBeInTheDocument();
      expect(screen.getByText('Milk')).toBeInTheDocument();
    });
  });

  it('highlights items expiring soon', async () => {
    render(<InventoryList />, { wrapper: createWrapper() });

    await waitFor(() => {
      const milkItem = screen.getByTestId('inventory-item-2');
      expect(milkItem).toHaveClass('bg-yellow-50'); // Warning background
    });
  });

  it('filters items by location', async () => {
    render(<InventoryList />, { wrapper: createWrapper() });

    const fridgeFilter = screen.getByText('Refrigerator');
    fireEvent.click(fridgeFilter);

    await waitFor(() => {
      expect(mockInventoryService.getItems).toHaveBeenCalledWith({
        location: 'refrigerator'
      });
    });
  });

  it('handles voice command for adding items', async () => {
    const mockVoiceCommand = jest.fn();
    render(<InventoryList onVoiceCommand={mockVoiceCommand} />, {
      wrapper: createWrapper()
    });

    const voiceButton = screen.getByLabelText('Voice add item');
    fireEvent.click(voiceButton);

    // Simulate voice recognition result
    fireEvent(window, new CustomEvent('voiceresult', {
      detail: { transcript: 'add 2 pounds of chicken to refrigerator' }
    }));

    await waitFor(() => {
      expect(mockVoiceCommand).toHaveBeenCalledWith(
        expect.objectContaining({
          action: 'add_item',
          item: expect.objectContaining({
            name: 'chicken',
            quantity: 2,
            unit: 'pounds',
            location: 'refrigerator'
          })
        })
      );
    });
  });
});
```

### Backend API Test

```typescript
import { NextRequest } from 'next/server';
import { GET, POST } from '@/app/api/inventory/route';
import { InventoryService } from '@/lib/services/inventory-service';
import { auth } from '@/lib/auth';

// Mock dependencies
jest.mock('@/lib/services/inventory-service');
jest.mock('@/lib/auth');

const mockInventoryService = InventoryService as jest.Mocked<
  typeof InventoryService
>;
const mockAuth = auth as jest.MockedFunction<typeof auth>;

describe('/api/inventory', () => {
  const mockUser = {
    id: 'user-123',
    householdId: 'household-456',
    email: 'test@example.com',
  };

  beforeEach(() => {
    mockAuth.mockResolvedValue({
      user: mockUser,
      expires: new Date(Date.now() + 3600000).toISOString(),
    });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('GET /api/inventory', () => {
    it('returns inventory items for authenticated user', async () => {
      const mockItems = [
        {
          id: '1',
          name: 'Tomatoes',
          quantity: 3,
          unit: 'pieces',
          category: 'vegetables',
          location: 'refrigerator',
          householdId: 'household-456',
        },
      ];

      mockInventoryService.getHouseholdItems.mockResolvedValue(mockItems);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data).toEqual(mockItems);
      expect(mockInventoryService.getHouseholdItems).toHaveBeenCalledWith(
        'household-456',
        {}
      );
    });

    it('filters items by location', async () => {
      mockInventoryService.getHouseholdItems.mockResolvedValue([]);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory?location=pantry'
      );
      const response = await GET(request);

      expect(response.status).toBe(200);
      expect(mockInventoryService.getHouseholdItems).toHaveBeenCalledWith(
        'household-456',
        { location: 'pantry' }
      );
    });

    it('returns 401 for unauthenticated requests', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(401);
    });
  });

  describe('POST /api/inventory', () => {
    it('creates new inventory item', async () => {
      const newItem = {
        name: 'Chicken Breast',
        quantity: 2,
        unit: 'pounds',
        category: 'proteins',
        location: 'refrigerator',
        expirationDate: '2025-09-20',
      };

      const createdItem = {
        id: 'item-789',
        ...newItem,
        expirationDate: new Date('2025-09-20'),
        householdId: 'household-456',
        addedBy: 'user-123',
      };

      mockInventoryService.createItem.mockResolvedValue(createdItem);

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(newItem),
        headers: { 'Content-Type': 'application/json' },
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(201);
      expect(data).toEqual(createdItem);
      expect(mockInventoryService.createItem).toHaveBeenCalledWith({
        ...newItem,
        expirationDate: new Date('2025-09-20'),
        householdId: 'household-456',
        addedBy: 'user-123',
      });
    });

    it('validates request data', async () => {
      const invalidItem = {
        name: '', // Invalid: empty name
        quantity: -1, // Invalid: negative quantity
        unit: 'pounds',
        category: 'invalid-category', // Invalid: not in enum
        location: 'refrigerator',
      };

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(invalidItem),
        headers: { 'Content-Type': 'application/json' },
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toContain('validation');
    });
  });
});
```

### E2E Test

```typescript
import { test, expect } from '@playwright/test';

test.describe('Cooking Mode', () => {
  test.beforeEach(async ({ page }) => {
    // Login user
    await page.goto('/login');
    await page.fill('[data-testid=email]', 'test@example.com');
    await page.fill('[data-testid=password]', 'password123');
    await page.click('[data-testid=login-button]');
    await expect(page).toHaveURL('/dashboard');
  });

  test('should start cooking mode and navigate through recipe steps', async ({
    page,
  }) => {
    // Navigate to recipe
    await page.goto('/recipes/classic-spaghetti-carbonara');

    // Start cooking mode
    await page.click('[data-testid=start-cooking]');
    await expect(page).toHaveURL('/cooking/classic-spaghetti-carbonara');

    // Verify cooking interface
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 1'
    );
    await expect(page.locator('[data-testid=step-instruction]')).toContainText(
      'Bring a large pot of salted water to boil'
    );

    // Check ingredients panel
    await expect(page.locator('[data-testid=ingredients-panel]')).toBeVisible();
    await expect(page.locator('[data-testid=ingredient-item]')).toHaveCount(6);

    // Navigate to next step
    await page.click('[data-testid=next-step]');
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 2'
    );

    // Test timer functionality
    await page.click('[data-testid=start-timer]');
    await expect(page.locator('[data-testid=active-timer]')).toBeVisible();
    await expect(page.locator('[data-testid=timer-display]')).toContainText(
      '10:00'
    );

    // Test voice commands (if supported)
    const voiceButton = page.locator('[data-testid=voice-button]');
    if (await voiceButton.isVisible()) {
      await voiceButton.click();
      await expect(page.locator('[data-testid=voice-status]')).toContainText(
        'Listening'
      );

      // Simulate voice command (this would be integration with actual voice API in real test)
      await page.evaluate(() => {
        window.dispatchEvent(
          new CustomEvent('voicecommand', {
            detail: { command: 'next step' },
          })
        );
      });

      await expect(page.locator('[data-testid=cooking-step]')).toContainText(
        'Step 3'
      );
    }

    // Complete recipe
    await page.click('[data-testid=next-step]'); // Step 3
    await page.click('[data-testid=next-step]'); // Step 4
    await page.click('[data-testid=next-step]'); // Step 5
    await page.click('[data-testid=complete-recipe]');

    // Verify completion screen
    await expect(
      page.locator('[data-testid=completion-message]')
    ).toContainText('Recipe completed!');
    await expect(page.locator('[data-testid=rating-prompt]')).toBeVisible();

    // Rate recipe
    await page.click('[data-testid=rating-star-5]');
    await page.fill(
      '[data-testid=review-text]',
      'Delicious and easy to follow!'
    );
    await page.click('[data-testid=submit-rating]');

    // Verify redirect to recipe page
    await expect(page).toHaveURL('/recipes/classic-spaghetti-carbonara');
    await expect(page.locator('[data-testid=user-rating]')).toContainText(
      '5 stars'
    );
  });

  test('should handle offline cooking mode', async ({ page, context }) => {
    // Start cooking mode while online
    await page.goto('/recipes/classic-spaghetti-carbonara');
    await page.click('[data-testid=start-cooking]');

    // Verify recipe data is cached
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 1'
    );

    // Go offline
    await context.setOffline(true);

    // Verify cooking mode still works
    await page.click('[data-testid=next-step]');
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 2'
    );

    // Start timer offline
    await page.click('[data-testid=start-timer]');
    await expect(page.locator('[data-testid=active-timer]')).toBeVisible();

    // Verify offline indicator
    await expect(page.locator('[data-testid=offline-indicator]')).toBeVisible();

    // Go back online
    await context.setOffline(false);

    // Verify sync happens
    await expect(
      page.locator('[data-testid=offline-indicator]')
    ).not.toBeVisible();
  });

  test('should support voice commands during cooking', async ({ page }) => {
    // Grant microphone permissions (in real test environment)
    await page.context().grantPermissions(['microphone']);

    await page.goto('/cooking/classic-spaghetti-carbonara');

    // Test voice activation
    await page.click('[data-testid=voice-button]');
    await expect(page.locator('[data-testid=voice-status]')).toContainText(
      'Listening'
    );

    // Simulate various voice commands
    const commands = [
      { command: 'next step', expectedAction: 'step advancement' },
      { command: 'previous step', expectedAction: 'step regression' },
      { command: 'set timer for 5 minutes', expectedAction: 'timer creation' },
      { command: 'pause timer', expectedAction: 'timer pause' },
      { command: 'repeat instructions', expectedAction: 'instruction repeat' },
    ];

    for (const { command, expectedAction } of commands) {
      await page.evaluate(cmd => {
        window.dispatchEvent(
          new CustomEvent('voicecommand', {
            detail: { command: cmd },
          })
        );
      }, command);

      // Verify appropriate action was taken
      // (This would be more specific based on actual implementation)
      await expect(page.locator('[data-testid=voice-feedback]')).toContainText(
        'Command processed'
      );
    }
  });
});
```
