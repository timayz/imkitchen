// API Response types
export interface ApiResponse<T = unknown> {
  data?: T;
  error?: string;
  message?: string;
  statusCode: number;
  timestamp: string;
  path?: string;
}

export interface ApiError {
  error: string;
  message: string;
  statusCode: number;
  timestamp: string;
  path?: string;
  details?: Record<string, unknown>;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

// API Request types
export interface PaginationParams {
  page?: number;
  limit?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface SearchParams extends PaginationParams {
  query?: string;
  filters?: Record<string, any>;
}

// HTTP Methods
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

// API Client configuration
export interface ApiClientConfig {
  baseURL: string;
  timeout: number;
  retries: number;
  retryDelay: number;
  headers?: Record<string, string>;
}

// Request configuration
export interface RequestConfig {
  method: HttpMethod;
  url: string;
  data?: Record<string, unknown>;
  params?: Record<string, any>;
  headers?: Record<string, string>;
  timeout?: number;
  withCredentials?: boolean;
}

// Authentication API types
export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: {
    id: string;
    email: string;
    name: string;
    householdId: string;
  };
  token: string;
  expiresAt: string;
}

export interface RegisterRequest {
  email: string;
  name: string;
  password: string;
  householdName: string;
  dietaryPreferences?: string[];
  allergies?: string[];
  language?: string;
  timezone?: string;
}

export interface RegisterResponse {
  user: {
    id: string;
    email: string;
    name: string;
    householdId: string;
  };
  household: {
    id: string;
    name: string;
  };
  token: string;
  expiresAt: string;
}

// User API types
export interface UpdateUserRequest {
  name?: string;
  dietaryPreferences?: string[];
  allergies?: string[];
  language?: string;
  timezone?: string;
}

export interface ChangePasswordRequest {
  currentPassword: string;
  newPassword: string;
}

export interface UserProfileResponse {
  id: string;
  email: string;
  name: string;
  dietaryPreferences: string[];
  allergies: string[];
  language: string;
  timezone: string;
  household: {
    id: string;
    name: string;
    memberCount: number;
  };
  stats: {
    joinedAt: string;
    lastActivityAt: string | null;
    activeSessions: number;
  };
}

// Household API types
export interface CreateHouseholdRequest {
  name: string;
  settings?: Record<string, any>;
}

export interface UpdateHouseholdRequest {
  name?: string;
  settings?: Record<string, any>;
}

export interface HouseholdResponse {
  id: string;
  name: string;
  memberCount: number;
  createdAt: string;
  settings: Record<string, any>;
  members: {
    id: string;
    name: string;
    email: string;
    role: 'owner' | 'member';
    joinedAt: string;
    lastActivity: string | null;
  }[];
  stats: {
    activeMembersCount: number;
    totalSessions: number;
  };
}

export interface InviteMemberRequest {
  email: string;
  role?: 'member';
}

export interface TransferOwnershipRequest {
  newOwnerId: string;
}

// Database health API types
export interface HealthCheckResponse {
  status: 'healthy' | 'unhealthy';
  message: string;
  timestamp: string;
  checks: {
    database: {
      status: 'healthy' | 'unhealthy';
      responseTime?: number;
      error?: string;
    };
    redis?: {
      status: 'healthy' | 'unhealthy';
      responseTime?: number;
      error?: string;
    };
  };
}

// Error response types
export enum ApiErrorCode {
  // Authentication errors
  UNAUTHORIZED = 'UNAUTHORIZED',
  FORBIDDEN = 'FORBIDDEN',
  INVALID_CREDENTIALS = 'INVALID_CREDENTIALS',
  TOKEN_EXPIRED = 'TOKEN_EXPIRED',
  
  // Validation errors
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  INVALID_INPUT = 'INVALID_INPUT',
  REQUIRED_FIELD_MISSING = 'REQUIRED_FIELD_MISSING',
  
  // Resource errors
  NOT_FOUND = 'NOT_FOUND',
  ALREADY_EXISTS = 'ALREADY_EXISTS',
  CONFLICT = 'CONFLICT',
  
  // Database errors
  DATABASE_ERROR = 'DATABASE_ERROR',
  CONNECTION_ERROR = 'CONNECTION_ERROR',
  CONSTRAINT_VIOLATION = 'CONSTRAINT_VIOLATION',
  
  // Server errors
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',
  TIMEOUT = 'TIMEOUT',
  
  // Rate limiting
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  
  // Business logic errors
  INSUFFICIENT_PERMISSIONS = 'INSUFFICIENT_PERMISSIONS',
  OPERATION_NOT_ALLOWED = 'OPERATION_NOT_ALLOWED',
  RESOURCE_LIMIT_EXCEEDED = 'RESOURCE_LIMIT_EXCEEDED',
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
  value?: unknown;
}

export interface DetailedApiError extends ApiError {
  code: ApiErrorCode;
  validationErrors?: ValidationError[];
  retryAfter?: number;
  requestId?: string;
}

// API Client types
export interface ApiClient {
  get<T = any>(url: string, config?: Partial<RequestConfig>): Promise<T>;
  post<T = unknown>(url: string, data?: unknown, config?: Partial<RequestConfig>): Promise<T>;
  put<T = unknown>(url: string, data?: unknown, config?: Partial<RequestConfig>): Promise<T>;
  patch<T = unknown>(url: string, data?: unknown, config?: Partial<RequestConfig>): Promise<T>;
  delete<T = any>(url: string, config?: Partial<RequestConfig>): Promise<T>;
}

// Request interceptor types
export interface RequestInterceptor {
  onFulfilled?: (config: RequestConfig) => RequestConfig | Promise<RequestConfig>;
  onRejected?: (error: unknown) => unknown;
}

export interface ResponseInterceptor {
  onFulfilled?: (response: unknown) => unknown;
  onRejected?: (error: unknown) => unknown;
}

// Retry configuration
export interface RetryConfig {
  retries: number;
  retryDelay: number;
  retryCondition?: (error: unknown) => boolean;
  onRetry?: (retryCount: number, error: unknown) => void;
}

// Rate limiting
export interface RateLimitConfig {
  maxRequests: number;
  windowMs: number;
  message?: string;
  standardHeaders?: boolean;
  legacyHeaders?: boolean;
}

// API versioning
export interface ApiVersion {
  version: string;
  deprecated?: boolean;
  sunsetDate?: string;
  replacedBy?: string;
}

// Request context
export interface RequestContext {
  requestId: string;
  userId?: string;
  householdId?: string;
  userAgent?: string;
  ipAddress?: string;
  timestamp: Date;
}

// Webhook types
export interface WebhookEvent {
  id: string;
  type: string;
  data: unknown;
  timestamp: string;
  version: string;
}

export interface WebhookPayload {
  event: WebhookEvent;
  signature: string;
}

// API documentation types
export interface ApiEndpoint {
  method: HttpMethod;
  path: string;
  description: string;
  parameters?: {
    name: string;
    type: string;
    required: boolean;
    description: string;
  }[];
  responses: {
    status: number;
    description: string;
    schema?: Record<string, unknown>;
  }[];
  examples?: {
    request?: Record<string, unknown>;
    response?: Record<string, unknown>;
  };
}