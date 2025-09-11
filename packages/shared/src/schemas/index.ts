// Shared Zod validation schemas
import { z } from 'zod';

export const createRecipeSchema = z.object({
  title: z.string().min(1).max(200),
  description: z.string().optional(),
  ingredients: z.array(z.any()),
  instructions: z.array(z.any()),
  prepTimeMinutes: z.number().min(0),
  cookTimeMinutes: z.number().min(0),
});

export const createUserSchema = z.object({
  email: z.string().email(),
  name: z.string().optional(),
});