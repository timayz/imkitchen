import { z } from "zod";
import { initTRPC } from '@trpc/server';

const t = initTRPC.context<{
  session: any;
  prisma: any;
}>().create();

export const router = t.router;
export const publicProcedure = t.procedure;

const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  name: z.string().min(1),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"],
});

export const simpleRouter = router({
  hello: publicProcedure
    .input(z.object({ name: z.string() }).optional())
    .query(({ input }) => {
      return {
        greeting: `Hello ${input?.name ?? 'World'}!`
      };
    }),
    
  health: publicProcedure
    .query(() => {
      return {
        status: 'ok',
        timestamp: new Date().toISOString()
      };
    }),

  auth: router({
    register: publicProcedure
      .input(registerSchema)
      .mutation(async ({ input, ctx }) => {
        const { email, name, password } = input;
        
        // Mock implementation that simulates Story 1.2 behavior
        console.log('Registration attempt:', { email, name });
        
        return {
          success: true,
          message: "Registration successful! Please check your email to verify your account.",
          user: {
            id: "mock-user-id",
            email,
            name,
          },
        };
      }),

    verifyEmail: publicProcedure
      .input(z.object({ token: z.string() }))
      .mutation(async ({ input }) => {
        console.log('Email verification attempt:', input.token);
        return {
          success: true,
          message: "Email verified successfully! You can now log in.",
          user: {
            id: "mock-user-id",
            email: "user@example.com",
            name: "Mock User",
            emailVerified: new Date(),
          },
        };
      }),

    resendVerification: publicProcedure
      .input(z.object({ email: z.string().email() }))
      .mutation(async ({ input }) => {
        console.log('Resend verification for:', input.email);
        return {
          success: true,
          message: "Verification email sent! Please check your inbox.",
        };
      }),
  }),

  user: router({
    me: publicProcedure
      .query(async () => {
        // Mock current user for Story 1.2 functionality
        return {
          id: "mock-user-id",
          email: "user@example.com", 
          name: "Mock User",
          image: null,
          emailVerified: new Date(),
          preferences: null,
          createdAt: new Date(),
          lastActiveAt: new Date(),
        };
      }),

    updateProfile: publicProcedure
      .input(z.object({
        name: z.string().min(1),
        preferences: z.object({
          dietaryRestrictions: z.array(z.string()).optional(),
          cookingSkillLevel: z.enum(["beginner", "intermediate", "advanced"]).optional(),
          preferredCuisines: z.array(z.string()).optional(),
        }).optional(),
      }))
      .mutation(async ({ input }) => {
        console.log('Profile update:', input);
        return {
          success: true,
          message: "Profile updated successfully",
          user: {
            id: "mock-user-id",
            email: "user@example.com",
            name: input.name,
            preferences: input.preferences,
          },
        };
      }),

    changePassword: publicProcedure
      .input(z.object({
        currentPassword: z.string(),
        newPassword: z.string().min(8),
        confirmNewPassword: z.string(),
      }).refine((data) => data.newPassword === data.confirmNewPassword, {
        message: "Passwords don't match",
        path: ["confirmNewPassword"],
      }))
      .mutation(async ({ input }) => {
        console.log('Password change attempt');
        return {
          success: true,
          message: "Password changed successfully",
        };
      }),
  }),
});

export type SimpleRouter = typeof simpleRouter;