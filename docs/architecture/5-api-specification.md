# 5. API Specification

## 5.1 tRPC Router Structure

```typescript
// Main App Router
export const appRouter = router({
  auth: authRouter,
  user: userRouter,
  recipe: recipeRouter,
  mealPlan: mealPlanRouter,
  notification: notificationRouter,
  shopping: shoppingRouter,
});

export type AppRouter = typeof appRouter;
```

## 5.2 Recipe Management API

```typescript
export const recipeRouter = router({
  // Get user's recipes with pagination and filtering
  list: protectedProcedure
    .input(z.object({
      page: z.number().default(1),
      limit: z.number().min(1).max(100).default(20),
      search: z.string().optional(),
      dietaryTags: z.array(z.string()).optional(),
      cuisine: z.string().optional(),
      difficulty: z.enum(['Easy', 'Medium', 'Hard']).optional(),
    }))
    .query(async ({ input, ctx }) => {
      const { page, limit, search, dietaryTags, cuisine, difficulty } = input;
      const skip = (page - 1) * limit;
      
      const where: Prisma.RecipeWhereInput = {
        userId: ctx.session.user.id,
        ...(search && {
          OR: [
            { title: { contains: search, mode: 'insensitive' } },
            { description: { contains: search, mode: 'insensitive' } }
          ]
        }),
        ...(dietaryTags && { dietaryTags: { hassome: dietaryTags } }),
        ...(cuisine && { cuisine }),
        ...(difficulty && { difficulty })
      };
      
      const [recipes, total] = await Promise.all([
        ctx.prisma.recipe.findMany({
          where,
          skip,
          take: limit,
          orderBy: { updatedAt: 'desc' },
          include: {
            _count: { select: { meals: true } }
          }
        }),
        ctx.prisma.recipe.count({ where })
      ]);
      
      return {
        recipes,
        pagination: {
          page,
          limit,
          total,
          totalPages: Math.ceil(total / limit)
        }
      };
    }),

  // Create new recipe
  create: protectedProcedure
    .input(createRecipeSchema)
    .mutation(async ({ input, ctx }) => {
      return await ctx.prisma.recipe.create({
        data: {
          ...input,
          userId: ctx.session.user.id
        }
      });
    }),

  // Import recipe from URL
  importFromUrl: protectedProcedure
    .input(z.object({ url: z.string().url() }))
    .mutation(async ({ input, ctx }) => {
      const recipeData = await parseRecipeFromUrl(input.url);
      return await ctx.prisma.recipe.create({
        data: {
          ...recipeData,
          userId: ctx.session.user.id,
          sourceUrl: input.url
        }
      });
    }),
});
```

## 5.3 Meal Planning API

```typescript
export const mealPlanRouter = router({
  // Create meal plan with "Fill My Week" automation
  create: protectedProcedure
    .input(z.object({
      title: z.string(),
      startDate: z.date(),
      endDate: z.date(),
      preferences: z.object({
        mealsPerDay: z.number().min(1).max(6),
        dietaryRestrictions: z.array(z.string()),
        cuisinePreferences: z.array(z.string()),
        skillLevel: z.enum(['beginner', 'intermediate', 'advanced']),
        timeConstraints: z.object({
          maxPrepTime: z.number(),
          maxCookTime: z.number()
        })
      }),
      autoFill: z.boolean().default(false)
    }))
    .mutation(async ({ input, ctx }) => {
      const mealPlan = await ctx.prisma.mealPlan.create({
        data: {
          title: input.title,
          startDate: input.startDate,
          endDate: input.endDate,
          userId: ctx.session.user.id
        }
      });

      if (input.autoFill) {
        const suggestedMeals = await generateMealSuggestions(
          mealPlan.id,
          input.preferences,
          ctx.session.user.id
        );
        
        await ctx.prisma.meal.createMany({
          data: suggestedMeals.map(meal => ({
            ...meal,
            mealPlanId: mealPlan.id
          }))
        });
      }

      return mealPlan;
    }),

  // Calculate timing schedule for meal
  calculateTiming: protectedProcedure
    .input(z.object({
      mealId: z.string(),
      targetServingTime: z.date(),
      servings: z.number().optional()
    }))
    .mutation(async ({ input, ctx }) => {
      const meal = await ctx.prisma.meal.findUniqueOrThrow({
        where: { id: input.mealId },
        include: { recipe: true }
      });

      const timingSchedule = await calculateOptimalTiming(
        meal.recipe,
        input.targetServingTime,
        input.servings || meal.servings
      );

      await ctx.prisma.meal.update({
        where: { id: input.mealId },
        data: {
          targetServingTime: input.targetServingTime,
          timingSchedule: timingSchedule as any,
          servings: input.servings || meal.servings
        }
      });

      // Schedule notifications
      await scheduleTimingNotifications(input.mealId, timingSchedule);

      return timingSchedule;
    }),
});
```

## 5.4 Notification API

```typescript
export const notificationRouter = router({
  // Subscribe to push notifications
  subscribe: protectedProcedure
    .input(z.object({
      endpoint: z.string(),
      keys: z.object({
        p256dh: z.string(),
        auth: z.string()
      })
    }))
    .mutation(async ({ input, ctx }) => {
      await ctx.prisma.user.update({
        where: { id: ctx.session.user.id },
        data: {
          notificationSettings: {
            pushSubscription: input
          }
        }
      });
    }),

  // Send test notification
  sendTest: protectedProcedure
    .mutation(async ({ ctx }) => {
      const notification = {
        id: generateId(),
        userId: ctx.session.user.id,
        type: 'test',
        title: 'Test Notification',
        body: 'Your notifications are working!',
        scheduledTime: new Date()
      };

      await sendNotification(notification);
      return { success: true };
    }),

  // Get notification history
  history: protectedProcedure
    .input(z.object({
      page: z.number().default(1),
      limit: z.number().default(20)
    }))
    .query(async ({ input, ctx }) => {
      return await ctx.prisma.timingNotification.findMany({
        where: { userId: ctx.session.user.id },
        orderBy: { scheduledTime: 'desc' },
        skip: (input.page - 1) * input.limit,
        take: input.limit
      });
    }),
});
```

## 5.5 Input/Output Validation Schemas

```typescript
// Recipe Schemas
export const createRecipeSchema = z.object({
  title: z.string().min(1).max(200),
  description: z.string().optional(),
  ingredients: z.array(z.object({
    name: z.string(),
    amount: z.number().positive(),
    unit: z.string(),
    notes: z.string().optional()
  })),
  instructions: z.array(z.object({
    step: z.number(),
    description: z.string(),
    duration: z.number().optional(),
    temperature: z.number().optional(),
    equipment: z.array(z.string()).optional()
  })),
  prepTimeMinutes: z.number().min(0),
  cookTimeMinutes: z.number().min(0),
  servings: z.number().min(1).max(20),
  difficulty: z.enum(['Easy', 'Medium', 'Hard']).optional(),
  cuisine: z.string().optional(),
  dietaryTags: z.array(z.string()),
  imageUrl: z.string().url().optional()
});

// Meal Plan Schemas
export const createMealPlanSchema = z.object({
  title: z.string().min(1).max(100),
  description: z.string().optional(),
  startDate: z.date(),
  endDate: z.date().refine((date, ctx) => {
    const startDate = ctx.parent.startDate;
    return date > startDate;
  }, 'End date must be after start date')
});

// Timing Schemas
export const timingScheduleSchema = z.object({
  targetServingTime: z.string().datetime(),
  totalDuration: z.number(),
  steps: z.array(z.object({
    id: z.string(),
    description: z.string(),
    startTime: z.string().datetime(),
    duration: z.number(),
    type: z.enum(['prep', 'cook', 'rest', 'serve']),
    dependencies: z.array(z.string()),
    notifications: z.object({
      beforeStart: z.array(z.number()),
      duringStep: z.array(z.number()),
      beforeEnd: z.array(z.number())
    })
  })),
  criticalPath: z.array(z.string())
});
```

## 5.6 Error Handling

```typescript
// Custom tRPC Error Types
export const AppError = {
  RECIPE_NOT_FOUND: new TRPCError({
    code: 'NOT_FOUND',
    message: 'Recipe not found or access denied'
  }),
  MEAL_PLAN_GENERATION_FAILED: new TRPCError({
    code: 'INTERNAL_SERVER_ERROR', 
    message: 'Failed to generate meal plan'
  }),
  NOTIFICATION_DELIVERY_FAILED: new TRPCError({
    code: 'INTERNAL_SERVER_ERROR',
    message: 'Failed to deliver notification'
  }),
  INVALID_TIMING_SCHEDULE: new TRPCError({
    code: 'BAD_REQUEST',
    message: 'Invalid timing schedule configuration'
  })
} as const;

// Error middleware
const errorHandler = (opts: { error: TRPCError; type: 'query' | 'mutation' }) => {
  const { error, type } = opts;
  
  // Log error with context
  logger.error('tRPC Error', {
    code: error.code,
    message: error.message,
    type,
    stack: error.stack
  });
  
  // Don't expose internal errors to client
  if (error.code === 'INTERNAL_SERVER_ERROR') {
    throw new TRPCError({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'An unexpected error occurred'
    });
  }
  
  throw error;
};
```
