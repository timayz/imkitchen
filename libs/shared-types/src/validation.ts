/**
 * Runtime validation utilities for shared types
 * Provides validation functions to ensure type safety at runtime
 */

import type {
  RecipeImportRequest,
  TagSuggestion,
  PopularTag,
  RecipeTag,
  TagValidationResult,
  RecipeAttribution,
  ContributorProfile,
  CommunityMetricsData,
  MetricsTimeframe,
  ContributorAchievement,
} from './rating.types';

// Validation error types
export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface ValidationResult<T> {
  isValid: boolean;
  data?: T;
  errors: ValidationError[];
}

// Type guards
export function isRecipeImportRequest(obj: any): obj is RecipeImportRequest {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.communityRecipeId === 'string' &&
    obj.communityRecipeId.length > 0 &&
    typeof obj.preserveAttribution === 'boolean' &&
    (obj.customizations === undefined || 
     (typeof obj.customizations === 'object' && obj.customizations !== null))
  );
}

export function isTagSuggestion(obj: any): obj is TagSuggestion {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.tag === 'string' &&
    typeof obj.confidence === 'number' &&
    typeof obj.usageCount === 'number' &&
    typeof obj.category === 'string' &&
    obj.tag.length > 0 &&
    obj.confidence >= 0 &&
    obj.confidence <= 1 &&
    obj.usageCount >= 0
  );
}

export function isPopularTag(obj: any): obj is PopularTag {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.tag === 'string' &&
    typeof obj.usageCount === 'number' &&
    typeof obj.category === 'string' &&
    typeof obj.trendingUp === 'boolean' &&
    obj.tag.length > 0 &&
    obj.usageCount >= 0
  );
}

export function isRecipeTag(obj: any): obj is RecipeTag {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.tag === 'string' &&
    typeof obj.voteCount === 'number' &&
    typeof obj.userVoted === 'boolean' &&
    typeof obj.confidence === 'number' &&
    obj.tag.length > 0 &&
    obj.confidence >= 0 &&
    obj.confidence <= 1
  );
}

export function isMetricsTimeframe(value: any): value is MetricsTimeframe {
  return typeof value === 'string' && 
    ['day', 'week', 'month', 'quarter', 'year', 'all'].includes(value);
}

export function isContributorAchievement(obj: any): obj is ContributorAchievement {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.id === 'string' &&
    typeof obj.title === 'string' &&
    typeof obj.description === 'string' &&
    typeof obj.emoji === 'string' &&
    obj.earnedAt instanceof Date &&
    typeof obj.category === 'string' &&
    typeof obj.points === 'number' &&
    obj.points >= 0
  );
}

// Validation functions with detailed error reporting
export function validateRecipeImportRequest(obj: any): ValidationResult<RecipeImportRequest> {
  const errors: ValidationError[] = [];

  if (!obj || typeof obj !== 'object') {
    return {
      isValid: false,
      errors: [{ field: 'root', message: 'Request must be an object', code: 'INVALID_TYPE' }]
    };
  }

  // Validate communityRecipeId
  if (!obj.communityRecipeId) {
    errors.push({
      field: 'communityRecipeId',
      message: 'Community recipe ID is required',
      code: 'REQUIRED_FIELD'
    });
  } else if (typeof obj.communityRecipeId !== 'string' || obj.communityRecipeId.trim().length === 0) {
    errors.push({
      field: 'communityRecipeId',
      message: 'Community recipe ID must be a non-empty string',
      code: 'INVALID_FORMAT'
    });
  }

  // Validate preserveAttribution
  if (typeof obj.preserveAttribution !== 'boolean') {
    errors.push({
      field: 'preserveAttribution',
      message: 'Preserve attribution must be a boolean',
      code: 'INVALID_TYPE'
    });
  }

  // Validate customizations (optional)
  if (obj.customizations !== undefined) {
    if (typeof obj.customizations !== 'object' || obj.customizations === null) {
      errors.push({
        field: 'customizations',
        message: 'Customizations must be an object',
        code: 'INVALID_TYPE'
      });
    } else {
      // Validate customization fields if provided
      if (obj.customizations.title !== undefined && typeof obj.customizations.title !== 'string') {
        errors.push({
          field: 'customizations.title',
          message: 'Title must be a string',
          code: 'INVALID_TYPE'
        });
      }
      
      if (obj.customizations.notes !== undefined && typeof obj.customizations.notes !== 'string') {
        errors.push({
          field: 'customizations.notes',
          message: 'Notes must be a string',
          code: 'INVALID_TYPE'
        });
      }
      
      if (obj.customizations.servingAdjustment !== undefined && 
          (typeof obj.customizations.servingAdjustment !== 'number' || obj.customizations.servingAdjustment <= 0)) {
        errors.push({
          field: 'customizations.servingAdjustment',
          message: 'Serving adjustment must be a positive number',
          code: 'INVALID_VALUE'
        });
      }
    }
  }

  return {
    isValid: errors.length === 0,
    data: errors.length === 0 ? obj as RecipeImportRequest : undefined,
    errors
  };
}

export function validateTagSuggestion(obj: any): ValidationResult<TagSuggestion> {
  const errors: ValidationError[] = [];

  if (!obj || typeof obj !== 'object') {
    return {
      isValid: false,
      errors: [{ field: 'root', message: 'Tag suggestion must be an object', code: 'INVALID_TYPE' }]
    };
  }

  // Validate tag
  if (!obj.tag || typeof obj.tag !== 'string' || obj.tag.trim().length === 0) {
    errors.push({
      field: 'tag',
      message: 'Tag must be a non-empty string',
      code: 'REQUIRED_FIELD'
    });
  } else if (obj.tag.length > 30) {
    errors.push({
      field: 'tag',
      message: 'Tag must be 30 characters or less',
      code: 'LENGTH_EXCEEDED'
    });
  }

  // Validate confidence
  if (typeof obj.confidence !== 'number') {
    errors.push({
      field: 'confidence',
      message: 'Confidence must be a number',
      code: 'INVALID_TYPE'
    });
  } else if (obj.confidence < 0 || obj.confidence > 1) {
    errors.push({
      field: 'confidence',
      message: 'Confidence must be between 0 and 1',
      code: 'INVALID_RANGE'
    });
  }

  // Validate usageCount
  if (typeof obj.usageCount !== 'number' || obj.usageCount < 0) {
    errors.push({
      field: 'usageCount',
      message: 'Usage count must be a non-negative number',
      code: 'INVALID_VALUE'
    });
  }

  // Validate category
  if (!obj.category || typeof obj.category !== 'string') {
    errors.push({
      field: 'category',
      message: 'Category must be a non-empty string',
      code: 'REQUIRED_FIELD'
    });
  }

  return {
    isValid: errors.length === 0,
    data: errors.length === 0 ? obj as TagSuggestion : undefined,
    errors
  };
}

export function validateTagArray(tags: any): ValidationResult<string[]> {
  const errors: ValidationError[] = [];

  if (!Array.isArray(tags)) {
    return {
      isValid: false,
      errors: [{ field: 'tags', message: 'Tags must be an array', code: 'INVALID_TYPE' }]
    };
  }

  if (tags.length === 0) {
    errors.push({
      field: 'tags',
      message: 'At least one tag is required',
      code: 'EMPTY_ARRAY'
    });
  }

  if (tags.length > 10) {
    errors.push({
      field: 'tags',
      message: 'Maximum 10 tags allowed',
      code: 'ARRAY_TOO_LARGE'
    });
  }

  tags.forEach((tag, index) => {
    if (typeof tag !== 'string') {
      errors.push({
        field: `tags[${index}]`,
        message: 'Each tag must be a string',
        code: 'INVALID_TYPE'
      });
    } else {
      const trimmed = tag.trim();
      if (trimmed.length === 0) {
        errors.push({
          field: `tags[${index}]`,
          message: 'Tag cannot be empty',
          code: 'EMPTY_VALUE'
        });
      } else if (trimmed.length > 30) {
        errors.push({
          field: `tags[${index}]`,
          message: 'Tag must be 30 characters or less',
          code: 'LENGTH_EXCEEDED'
        });
      } else if (!/^[a-z0-9_\-\s]+$/i.test(trimmed)) {
        errors.push({
          field: `tags[${index}]`,
          message: 'Tag contains invalid characters',
          code: 'INVALID_FORMAT'
        });
      }
    }
  });

  // Check for duplicates
  const uniqueTags = new Set(tags.map(tag => typeof tag === 'string' ? tag.trim().toLowerCase() : tag));
  if (uniqueTags.size !== tags.length) {
    errors.push({
      field: 'tags',
      message: 'Duplicate tags are not allowed',
      code: 'DUPLICATE_VALUES'
    });
  }

  return {
    isValid: errors.length === 0,
    data: errors.length === 0 ? tags.map(tag => tag.trim().toLowerCase()) : undefined,
    errors
  };
}

export function validateRecipeAttribution(obj: any): ValidationResult<RecipeAttribution> {
  const errors: ValidationError[] = [];

  if (!obj || typeof obj !== 'object') {
    return {
      isValid: false,
      errors: [{ field: 'root', message: 'Attribution must be an object', code: 'INVALID_TYPE' }]
    };
  }

  // Required string fields
  const requiredStringFields = ['id', 'recipeId', 'originalContributorId', 'originalContributor'];
  requiredStringFields.forEach(field => {
    if (!obj[field] || typeof obj[field] !== 'string') {
      errors.push({
        field,
        message: `${field} is required and must be a string`,
        code: 'REQUIRED_FIELD'
      });
    }
  });

  // Validate dates
  if (!(obj.importDate instanceof Date) && !isValidDateString(obj.importDate)) {
    errors.push({
      field: 'importDate',
      message: 'Import date must be a valid date',
      code: 'INVALID_DATE'
    });
  }

  // Validate boolean fields
  if (typeof obj.preserveAttribution !== 'boolean') {
    errors.push({
      field: 'preserveAttribution',
      message: 'Preserve attribution must be a boolean',
      code: 'INVALID_TYPE'
    });
  }

  // Validate arrays
  if (!Array.isArray(obj.customizations)) {
    errors.push({
      field: 'customizations',
      message: 'Customizations must be an array',
      code: 'INVALID_TYPE'
    });
  }

  if (!Array.isArray(obj.recipeChain)) {
    errors.push({
      field: 'recipeChain',
      message: 'Recipe chain must be an array',
      code: 'INVALID_TYPE'
    });
  }

  // Validate nested objects
  if (!obj.communityMetrics || typeof obj.communityMetrics !== 'object') {
    errors.push({
      field: 'communityMetrics',
      message: 'Community metrics must be an object',
      code: 'INVALID_TYPE'
    });
  }

  return {
    isValid: errors.length === 0,
    data: errors.length === 0 ? obj as RecipeAttribution : undefined,
    errors
  };
}

// Helper validation functions
export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function validateURL(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

export function validateUUID(uuid: string): boolean {
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(uuid);
}

function isValidDateString(dateString: any): boolean {
  if (typeof dateString !== 'string') return false;
  const date = new Date(dateString);
  return !isNaN(date.getTime());
}

// Sanitization functions
export function sanitizeTag(tag: string): string {
  return tag.trim().toLowerCase().replace(/[^a-z0-9_\-\s]/gi, '');
}

export function sanitizeString(str: string, maxLength: number = 255): string {
  return str.trim().slice(0, maxLength);
}

// Batch validation utilities
export function validateBatch<T>(
  items: any[],
  validator: (item: any) => ValidationResult<T>
): ValidationResult<T[]> {
  const errors: ValidationError[] = [];
  const validItems: T[] = [];

  if (!Array.isArray(items)) {
    return {
      isValid: false,
      errors: [{ field: 'items', message: 'Input must be an array', code: 'INVALID_TYPE' }]
    };
  }

  items.forEach((item, index) => {
    const result = validator(item);
    if (result.isValid && result.data) {
      validItems.push(result.data);
    } else {
      result.errors.forEach(error => {
        errors.push({
          ...error,
          field: `items[${index}].${error.field}`,
        });
      });
    }
  });

  return {
    isValid: errors.length === 0,
    data: errors.length === 0 ? validItems : undefined,
    errors
  };
}

// Validation schemas for common patterns
export const ValidationSchemas = {
  recipeId: (value: any) => validateUUID(value) ? [] : [{ field: 'recipeId', message: 'Invalid recipe ID format', code: 'INVALID_UUID' }],
  userId: (value: any) => validateUUID(value) ? [] : [{ field: 'userId', message: 'Invalid user ID format', code: 'INVALID_UUID' }],
  tag: (value: any) => {
    const errors: ValidationError[] = [];
    if (typeof value !== 'string') {
      errors.push({ field: 'tag', message: 'Tag must be a string', code: 'INVALID_TYPE' });
    } else {
      const sanitized = sanitizeTag(value);
      if (sanitized.length === 0) {
        errors.push({ field: 'tag', message: 'Tag cannot be empty', code: 'EMPTY_VALUE' });
      } else if (sanitized.length > 30) {
        errors.push({ field: 'tag', message: 'Tag too long', code: 'LENGTH_EXCEEDED' });
      }
    }
    return errors;
  },
  timeframe: (value: any) => isMetricsTimeframe(value) ? [] : [{ field: 'timeframe', message: 'Invalid timeframe', code: 'INVALID_ENUM' }],
};

// Error message helpers
export function formatValidationErrors(errors: ValidationError[]): string {
  return errors.map(error => `${error.field}: ${error.message}`).join('; ');
}

export function getErrorsByField(errors: ValidationError[]): Record<string, ValidationError[]> {
  return errors.reduce((acc, error) => {
    if (!acc[error.field]) {
      acc[error.field] = [];
    }
    acc[error.field].push(error);
    return acc;
  }, {} as Record<string, ValidationError[]>);
}