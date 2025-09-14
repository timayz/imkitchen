import {
  User,
  Household,
  Session,
  DietaryPreference,
  Language,
} from '@prisma/client';

// Authentication types
export interface AuthUser {
  id: string;
  email: string;
  name: string;
  householdId: string;
  language: Language;
  timezone: string;
}

export interface AuthSession {
  id: string;
  userId: string;
  token: string;
  expiresAt: Date;
  user?: AuthUser;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterData {
  email: string;
  name: string;
  password: string;
  householdName: string;
  dietaryPreferences?: DietaryPreference[];
  allergies?: string[];
  language?: Language;
  timezone?: string;
}

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  dietaryPreferences: DietaryPreference[];
  allergies: string[];
  language: Language;
  timezone: string;
  householdId: string;
  household: {
    id: string;
    name: string;
    memberCount: number;
    settings: Record<string, unknown>;
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface UpdateProfileData {
  name?: string;
  dietaryPreferences?: DietaryPreference[];
  allergies?: string[];
  language?: Language;
  timezone?: string;
}

export interface ChangePasswordData {
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}

// JWT token payload
export interface JWTPayload {
  userId: string;
  email: string;
  householdId: string;
  iat: number;
  exp: number;
}

// Authentication context
export interface AuthContext {
  user: AuthUser | null;
  session: AuthSession | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

// Authentication actions
export interface AuthActions {
  login: (credentials: LoginCredentials) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => Promise<void>;
  updateProfile: (data: UpdateProfileData) => Promise<void>;
  changePassword: (data: ChangePasswordData) => Promise<void>;
  refreshSession: () => Promise<void>;
}

// Session management
export interface SessionData {
  user: AuthUser;
  expiresAt: Date;
  createdAt: Date;
}

export interface CreateSessionData {
  userId: string;
  expiresAt?: Date;
  userAgent?: string;
  ipAddress?: string;
}

// Authentication errors
export enum AuthError {
  INVALID_CREDENTIALS = 'INVALID_CREDENTIALS',
  USER_NOT_FOUND = 'USER_NOT_FOUND',
  EMAIL_ALREADY_EXISTS = 'EMAIL_ALREADY_EXISTS',
  INVALID_TOKEN = 'INVALID_TOKEN',
  SESSION_EXPIRED = 'SESSION_EXPIRED',
  UNAUTHORIZED = 'UNAUTHORIZED',
  FORBIDDEN = 'FORBIDDEN',
  ACCOUNT_DISABLED = 'ACCOUNT_DISABLED',
  PASSWORD_REQUIREMENTS_NOT_MET = 'PASSWORD_REQUIREMENTS_NOT_MET',
  HOUSEHOLD_NOT_FOUND = 'HOUSEHOLD_NOT_FOUND',
  MAX_SESSIONS_EXCEEDED = 'MAX_SESSIONS_EXCEEDED',
}

// Password requirements
export interface PasswordRequirements {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  maxLength: number;
}

// Default password requirements
export const DEFAULT_PASSWORD_REQUIREMENTS: PasswordRequirements = {
  minLength: 8,
  requireUppercase: true,
  requireLowercase: true,
  requireNumbers: true,
  requireSpecialChars: false,
  maxLength: 128,
};

// Type guards
export function isAuthUser(obj: unknown): obj is AuthUser {
  return (
    !!obj &&
    typeof obj === 'object' &&
    'id' in obj &&
    typeof (obj as Record<string, unknown>).id === 'string' &&
    'email' in obj &&
    typeof (obj as Record<string, unknown>).email === 'string' &&
    'name' in obj &&
    typeof (obj as Record<string, unknown>).name === 'string' &&
    'householdId' in obj &&
    typeof (obj as Record<string, unknown>).householdId === 'string'
  );
}

export function isValidSession(session: AuthSession): boolean {
  return session.expiresAt > new Date();
}

// Authentication utilities
export interface AuthUtils {
  hashPassword: (password: string) => Promise<string>;
  comparePassword: (password: string, hash: string) => Promise<boolean>;
  generateToken: (payload: JWTPayload) => string;
  verifyToken: (token: string) => JWTPayload | null;
  generateSessionToken: () => string;
  validatePassword: (
    password: string,
    requirements?: PasswordRequirements
  ) => boolean;
  sanitizeUser: (user: User) => AuthUser;
}

// Database types with relationships
export type UserWithHousehold = User & {
  household: Household;
};

export type UserWithSessions = User & {
  sessions: Session[];
};

export type HouseholdWithUsers = Household & {
  users: User[];
  _count: {
    users: number;
  };
};

export type SessionWithUser = Session & {
  user: User;
};
