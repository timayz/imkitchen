import { z } from 'zod';
import { DietaryPreference, Language } from '@prisma/client';

// Password validation schema
export const passwordSchema = z
  .string()
  .min(8, 'Password must be at least 8 characters long')
  .max(128, 'Password must be less than 128 characters')
  .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
  .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
  .regex(/[0-9]/, 'Password must contain at least one number');

// Registration schema
export const registerSchema = z.object({
  email: z
    .string()
    .email('Please enter a valid email address')
    .max(255, 'Email must be less than 255 characters'),
  name: z
    .string()
    .min(2, 'Name must be at least 2 characters long')
    .max(100, 'Name must be less than 100 characters')
    .regex(
      /^[a-zA-Z\s'-]+$/,
      'Name can only contain letters, spaces, hyphens, and apostrophes'
    ),
  password: passwordSchema,
  householdName: z
    .string()
    .min(2, 'Household name must be at least 2 characters long')
    .max(100, 'Household name must be less than 100 characters'),
  dietaryPreferences: z
    .array(z.nativeEnum(DietaryPreference))
    .optional()
    .default([]),
  allergies: z
    .array(z.string().max(50, 'Allergy name must be less than 50 characters'))
    .optional()
    .default([]),
  language: z.nativeEnum(Language).optional().default(Language.en),
  timezone: z
    .string()
    .max(50, 'Timezone must be less than 50 characters')
    .optional()
    .default('UTC'),
});

// Login schema
export const loginSchema = z.object({
  email: z
    .string()
    .email('Please enter a valid email address')
    .max(255, 'Email must be less than 255 characters'),
  password: z.string().min(1, 'Password is required'),
});

// Update profile schema
export const updateProfileSchema = z.object({
  name: z
    .string()
    .min(2, 'Name must be at least 2 characters long')
    .max(100, 'Name must be less than 100 characters')
    .regex(
      /^[a-zA-Z\s'-]+$/,
      'Name can only contain letters, spaces, hyphens, and apostrophes'
    )
    .optional(),
  dietaryPreferences: z.array(z.nativeEnum(DietaryPreference)).optional(),
  allergies: z
    .array(z.string().max(50, 'Allergy name must be less than 50 characters'))
    .optional(),
  language: z.nativeEnum(Language).optional(),
  timezone: z
    .string()
    .max(50, 'Timezone must be less than 50 characters')
    .optional(),
});

// Change password schema
export const changePasswordSchema = z
  .object({
    currentPassword: z.string().min(1, 'Current password is required'),
    newPassword: passwordSchema,
    confirmPassword: z.string().min(1, 'Password confirmation is required'),
  })
  .refine(data => data.newPassword === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
  });

// Password reset request schema
export const passwordResetRequestSchema = z.object({
  email: z
    .string()
    .email('Please enter a valid email address')
    .max(255, 'Email must be less than 255 characters'),
});

// Password reset confirm schema
export const passwordResetConfirmSchema = z
  .object({
    token: z.string().min(1, 'Reset token is required'),
    password: passwordSchema,
    confirmPassword: z.string().min(1, 'Password confirmation is required'),
  })
  .refine(data => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
  });

// Type exports
export type RegisterData = z.infer<typeof registerSchema>;
export type LoginData = z.infer<typeof loginSchema>;
export type UpdateProfileData = z.infer<typeof updateProfileSchema>;
export type ChangePasswordData = z.infer<typeof changePasswordSchema>;
export type PasswordResetRequestData = z.infer<
  typeof passwordResetRequestSchema
>;
export type PasswordResetConfirmData = z.infer<
  typeof passwordResetConfirmSchema
>;
