# Frontend Architecture

## Component Architecture

### Component Organization

```text
src/
├── components/
│   ├── ui/                    # Base UI components (buttons, inputs, etc.)
│   ├── forms/                 # Form components with validation
│   ├── inventory/             # Inventory-specific components
│   ├── recipes/               # Recipe-related components
│   ├── meal-planning/         # Meal planning components
│   ├── shopping/              # Shopping list components
│   ├── cooking/               # Cooking mode components
│   ├── voice/                 # Voice interaction components
│   └── layout/                # Layout and navigation components
├── app/                       # Next.js 14 app directory
│   ├── (auth)/               # Auth route group
│   ├── dashboard/            # Dashboard pages
│   ├── inventory/            # Inventory pages
│   ├── recipes/              # Recipe pages
│   ├── meal-planning/        # Meal planning pages
│   ├── shopping/             # Shopping pages
│   ├── cooking/              # Cooking mode pages
│   ├── api/                  # API routes
│   ├── globals.css           # Global styles
│   └── layout.tsx            # Root layout
├── hooks/                    # Custom React hooks
├── lib/                      # Utility functions and configurations
├── stores/                   # State management
├── types/                    # TypeScript type definitions
└── services/                 # API client services
```

### Component Template

```typescript
import { FC, ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ComponentProps {
  children?: ReactNode;
  className?: string;
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  onClick?: () => void;
}

const Component: FC<ComponentProps> = ({
  children,
  className,
  variant = 'primary',
  size = 'md',
  disabled = false,
  onClick,
  ...props
}) => {
  const baseClasses = 'inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50';
  
  const variantClasses = {
    primary: 'bg-orange-500 text-white hover:bg-orange-600',
    secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
    danger: 'bg-red-500 text-white hover:bg-red-600',
  };
  
  const sizeClasses = {
    sm: 'h-9 px-3 text-sm',
    md: 'h-11 px-4 text-base',
    lg: 'h-12 px-6 text-lg',
  };

  return (
    <button
      className={cn(
        baseClasses,
        variantClasses[variant],
        sizeClasses[size],
        className
      )}
      disabled={disabled}
      onClick={onClick}
      {...props}
    >
      {children}
    </button>
  );
};

export default Component;
```

## State Management Architecture

### State Structure

```typescript
// Global App State
interface AppState {
  user: {
    currentUser: User | null;
    household: Household | null;
    preferences: UserPreferences;
    isAuthenticated: boolean;
  };
  
  inventory: {
    items: InventoryItem[];
    categories: InventoryCategory[];
    locations: StorageLocation[];
    filters: InventoryFilters;
    loading: boolean;
    error: string | null;
  };
  
  recipes: {
    searchResults: Recipe[];
    favorites: Recipe[];
    recentlyViewed: Recipe[];
    currentRecipe: Recipe | null;
    suggestions: Recipe[];
    loading: boolean;
  };
  
  mealPlanning: {
    currentPlan: MealPlan | null;
    weeklyPlans: MealPlan[];
    calendarView: CalendarView;
    draggedRecipe: Recipe | null;
    loading: boolean;
  };
  
  shopping: {
    activeLists: ShoppingList[];
    currentList: ShoppingList | null;
    categories: StoreCategory[];
    loading: boolean;
  };
  
  cooking: {
    activeSession: CookingSession | null;
    currentStep: number;
    timers: Timer[];
    voiceActive: boolean;
    progress: CookingProgress;
  };
  
  voice: {
    isListening: boolean;
    isProcessing: boolean;
    lastCommand: string;
    error: string | null;
    supported: boolean;
  };
  
  ui: {
    theme: 'light' | 'dark' | 'system';
    language: Language;
    notifications: Notification[];
    modals: ModalState;
    navigation: NavigationState;
  };
}
```

### State Management Patterns

- **Context + useReducer for Global State** - Authentication, user preferences, household data
- **React Query for Server State** - API data caching, synchronization, background updates
- **useState for Local Component State** - Form inputs, UI interactions, temporary state
- **useCallback and useMemo for Performance** - Prevent unnecessary re-renders in cooking mode
- **Custom Hooks for Reusable Logic** - Voice commands, timer management, inventory operations

## Routing Architecture

### Route Organization

```text
app/
├── (auth)/
│   ├── login/
│   └── register/
├── dashboard/
│   └── page.tsx
├── inventory/
│   ├── page.tsx
│   ├── add/
│   └── [itemId]/
├── recipes/
│   ├── page.tsx
│   ├── search/
│   ├── favorites/
│   ├── [recipeId]/
│   └── create/
├── meal-planning/
│   ├── page.tsx
│   ├── calendar/
│   └── templates/
├── shopping/
│   ├── page.tsx
│   ├── lists/
│   └── [listId]/
├── cooking/
│   ├── [recipeId]/
│   └── mode/
├── settings/
│   ├── page.tsx
│   ├── profile/
│   ├── household/
│   └── preferences/
└── api/
    ├── auth/
    ├── inventory/
    ├── recipes/
    ├── meal-plans/
    ├── shopping/
    └── voice/
```

### Protected Route Pattern

```typescript
import { redirect } from 'next/navigation';
import { auth } from '@/lib/auth';

interface ProtectedLayoutProps {
  children: React.ReactNode;
}

export default async function ProtectedLayout({ children }: ProtectedLayoutProps) {
  const session = await auth();
  
  if (!session?.user) {
    redirect('/login');
  }

  return (
    <div className="min-h-screen bg-background">
      <Navigation user={session.user} />
      <main className="container mx-auto px-4 py-8">
        {children}
      </main>
    </div>
  );
}
```

## Frontend Services Layer

### API Client Setup

```typescript
import { z } from 'zod';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || '/api';

class ApiClient {
  private baseURL: string;
  
  constructor(baseURL: string = API_BASE_URL) {
    this.baseURL = baseURL;
  }
  
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;
    
    const config: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    };
    
    const response = await fetch(url, config);
    
    if (!response.ok) {
      const error = await response.json();
      throw new ApiError(error.message, response.status, error);
    }
    
    return response.json();
  }
  
  get<T>(endpoint: string, params?: Record<string, string>): Promise<T> {
    const searchParams = params ? `?${new URLSearchParams(params)}` : '';
    return this.request<T>(`${endpoint}${searchParams}`);
  }
  
  post<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }
  
  put<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }
  
  delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

export const apiClient = new ApiClient();

class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public details?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}
```

### Service Example

```typescript
import { apiClient } from '@/lib/api-client';
import { InventoryItem, InventoryItemCreate, InventoryItemUpdate } from '@/types/inventory';

export class InventoryService {
  static async getItems(filters?: {
    location?: string;
    category?: string;
    expiringSoon?: boolean;
  }): Promise<InventoryItem[]> {
    return apiClient.get<InventoryItem[]>('/inventory', filters);
  }
  
  static async createItem(item: InventoryItemCreate): Promise<InventoryItem> {
    return apiClient.post<InventoryItem>('/inventory', item);
  }
  
  static async updateItem(id: string, updates: InventoryItemUpdate): Promise<InventoryItem> {
    return apiClient.put<InventoryItem>(`/inventory/${id}`, updates);
  }
  
  static async deleteItem(id: string): Promise<void> {
    return apiClient.delete<void>(`/inventory/${id}`);
  }
  
  static async getExpiringItems(days: number = 7): Promise<InventoryItem[]> {
    return apiClient.get<InventoryItem[]>('/inventory/expiring', { days: days.toString() });
  }
}
```
