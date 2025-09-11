import { z } from 'zod';
import { router, publicProcedure } from '../lib/trpc';

export const recipeRouter = router({
  getAll: publicProcedure
    .query(async () => {
      // TODO: Implement with Prisma
      return [
        {
          id: '1',
          title: 'Sample Recipe',
          description: 'A sample recipe for testing',
          prepTimeMinutes: 15,
          cookTimeMinutes: 30,
        },
      ];
    }),

  getById: publicProcedure
    .input(z.object({ id: z.string() }))
    .query(async ({ input }) => {
      // TODO: Implement with Prisma
      return {
        id: input.id,
        title: 'Sample Recipe',
        description: 'A sample recipe for testing',
        prepTimeMinutes: 15,
        cookTimeMinutes: 30,
      };
    }),

  create: publicProcedure
    .input(z.object({
      title: z.string(),
      description: z.string().optional(),
      ingredients: z.array(z.any()),
      instructions: z.array(z.any()),
      prepTimeMinutes: z.number(),
      cookTimeMinutes: z.number(),
    }))
    .mutation(async ({ input }) => {
      // TODO: Implement with Prisma
      return {
        id: 'new-recipe-id',
        ...input,
      };
    }),
});