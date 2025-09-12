import { router } from '../lib/trpc';
import { authRouter } from './auth';
import { recipeRouter } from './recipe';
import { userRouter } from './user';

export const appRouter = router({
  auth: authRouter,
  recipe: recipeRouter,
  user: userRouter,
});

export type AppRouter = typeof appRouter;