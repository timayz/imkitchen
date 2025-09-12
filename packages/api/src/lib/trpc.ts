import { initTRPC, TRPCError } from '@trpc/server';
import { ZodError } from 'zod';
import { prisma } from './prisma';

// Define context type
export interface Context {
  session: any | null; // Simplified session type to avoid NextAuth imports
  prisma: typeof prisma;
}

// Create context function
export const createContext = async (): Promise<Context> => {
  // For development, session will be handled by individual procedures
  return {
    session: null, // Will be populated by individual procedures as needed
    prisma,
  };
};

const t = initTRPC.context<Context>().create({
  errorFormatter({ shape, error }) {
    return {
      ...shape,
      data: {
        ...shape.data,
        zodError:
          error.cause instanceof ZodError ? error.cause.flatten() : null,
      },
    };
  },
});

// Authentication middleware
const enforceUserIsAuthed = t.middleware(({ ctx, next }) => {
  if (!ctx.session || !ctx.session.user) {
    throw new TRPCError({ code: 'UNAUTHORIZED' });
  }
  return next({
    ctx: {
      ...ctx,
      session: { ...ctx.session, user: ctx.session.user },
    },
  });
});

export const router = t.router;
export const publicProcedure = t.procedure;
export const protectedProcedure = t.procedure.use(enforceUserIsAuthed);