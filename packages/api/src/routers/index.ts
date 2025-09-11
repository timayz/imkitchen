import { router } from '../lib/trpc';
import { recipeRouter } from './recipe';
import { userRouter } from './user';

export const appRouter = router({
  recipe: recipeRouter,
  user: userRouter,
});

export type AppRouter = typeof appRouter;