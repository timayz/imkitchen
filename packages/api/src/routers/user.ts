import { z } from 'zod';
import { router, publicProcedure } from '../lib/trpc';

export const userRouter = router({
  getProfile: publicProcedure
    .input(z.object({ id: z.string() }))
    .query(async ({ input }) => {
      // TODO: Implement with Prisma
      return {
        id: input.id,
        email: 'user@example.com',
        name: 'Sample User',
        createdAt: new Date(),
      };
    }),

  create: publicProcedure
    .input(z.object({
      email: z.string().email(),
      name: z.string().optional(),
    }))
    .mutation(async ({ input }) => {
      // TODO: Implement with Prisma
      return {
        id: 'new-user-id',
        ...input,
        createdAt: new Date(),
      };
    }),
});