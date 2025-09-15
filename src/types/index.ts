// Re-export all types for easy importing
export * from './auth';
export * from './api';

// Re-export Prisma generated types
export type {
  User,
  Household,
  Session,
  DietaryPreference,
  Language,
  Prisma,
} from '@prisma/client';

// Database operation result types
export interface DatabaseResult<T> {
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

export interface TransactionResult<T> {
  success: boolean;
  data?: T;
  error?: string;
  rollback?: boolean;
}

// Common utility types
export type ID = string;
export type UUID = string;
export type Timestamp = Date;
export type Email = string;
export type URL = string;

// Generic CRUD operation types
export interface CreateOperation<T> {
  data: T;
}

export interface UpdateOperation<T> {
  id: ID;
  data: Partial<T>;
}

export interface DeleteOperation {
  id: ID;
}

export interface FindOperation {
  id?: ID;
  where?: Record<string, unknown>;
  include?: Record<string, unknown>;
  orderBy?: Record<string, unknown>;
  take?: number;
  skip?: number;
}

// Service response types
export interface ServiceResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
  metadata?: {
    timestamp: Timestamp;
    requestId?: string;
    duration?: number;
  };
}

// Repository operation types
export interface RepositoryOptions {
  transaction?: Record<string, unknown>;
  include?: Record<string, unknown>;
  select?: Record<string, unknown>;
}

// Logger context types
export interface LogContext {
  userId?: ID;
  householdId?: ID;
  operation?: string;
  duration?: number;
  error?: string;
  metadata?: Record<string, unknown>;
}

// Configuration types
export interface DatabaseConfig {
  url: string;
  connectionLimit: number;
  connectionTimeout: number;
  idleTimeout: number;
  enableLogging: boolean;
  logLevel: string;
}

export interface AppConfig {
  env: string;
  logLevel: string;
  publicUrl: string;
  apiUrl: string;
  features: {
    enableVoiceCommands: boolean;
    enableRecipeImport: boolean;
    enableEmailNotifications: boolean;
    enableFileUploads: boolean;
  };
}

// Event types for future use
export interface DomainEvent {
  id: ID;
  type: string;
  aggregateId: ID;
  aggregateType: string;
  data: Record<string, unknown>;
  metadata: {
    timestamp: Timestamp;
    version: number;
    causationId?: ID;
    correlationId?: ID;
  };
}

// Validation types
export interface ValidationRule {
  field: string;
  rules: string[];
  message?: string;
}

export interface ValidationResult {
  isValid: boolean;
  errors: {
    field: string;
    message: string;
    code: string;
  }[];
}

// Cache types
export interface CacheEntry<T> {
  key: string;
  value: T;
  expiresAt: Timestamp;
  createdAt: Timestamp;
}

export interface CacheOptions {
  ttl?: number; // Time to live in seconds
  tags?: string[];
  namespace?: string;
}

// Monitoring and metrics types
export interface PerformanceMetric {
  operation: string;
  duration: number;
  timestamp: Timestamp;
  success: boolean;
  metadata?: Record<string, unknown>;
}

export interface HealthCheck {
  component: string;
  status: 'healthy' | 'unhealthy' | 'degraded';
  message?: string;
  responseTime?: number;
  timestamp: Timestamp;
}

// Security types
export interface SecurityContext {
  userId?: ID;
  householdId?: ID;
  permissions: string[];
  roles: string[];
  ipAddress?: string;
  userAgent?: string;
}

// Feature flag types
export interface FeatureFlag {
  name: string;
  enabled: boolean;
  rolloutPercentage?: number;
  conditions?: {
    userId?: ID[];
    householdId?: ID[];
    userProperties?: Record<string, unknown>;
  };
}

// Notification types (for future use)
export interface Notification {
  id: ID;
  userId: ID;
  type: string;
  title: string;
  message: string;
  data?: Record<string, unknown>;
  read: boolean;
  createdAt: Timestamp;
  expiresAt?: Timestamp;
}

// File upload types (for future use)
export interface FileUpload {
  id: ID;
  filename: string;
  originalName: string;
  mimeType: string;
  size: number;
  url: string;
  uploadedBy: ID;
  uploadedAt: Timestamp;
}

// Search types (for future use)
export interface SearchQuery {
  query: string;
  filters?: Record<string, unknown>;
  facets?: string[];
  pagination?: {
    page: number;
    limit: number;
  };
  sorting?: {
    field: string;
    direction: 'asc' | 'desc';
  }[];
}

export interface SearchResult<T> {
  items: T[];
  total: number;
  facets?: Record<string, { value: string; count: number }[]>;
  pagination: {
    page: number;
    limit: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

// Export helper type utilities
export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>;
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};
export type NonNullable<T> = T extends null | undefined ? never : T;
