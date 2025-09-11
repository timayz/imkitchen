// Shared TypeScript types
export interface User {
  id: string;
  email: string;
  name?: string;
  createdAt: Date;
}

export interface Recipe {
  id: string;
  title: string;
  description?: string;
  ingredients: any[];
  instructions: any[];
  prepTimeMinutes: number;
  cookTimeMinutes: number;
  userId: string;
  createdAt: Date;
}