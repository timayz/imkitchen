// Jest globals are automatically available in the test environment
import { 
  calculatePasswordStrength, 
  getPasswordStrengthColor, 
  getPasswordStrengthText 
} from './password';

describe('Password Utilities', () => {
  describe('calculatePasswordStrength', () => {
    it('should return weak score for short passwords', () => {
      const result = calculatePasswordStrength('123');
      expect(result.score).toBe(1); // Only numbers
      expect(result.isValid).toBe(false);
      expect(result.feedback).toContain('Password must be at least 8 characters long');
    });

    it('should return weak score for passwords without variety', () => {
      const result = calculatePasswordStrength('password');
      expect(result.score).toBe(2); // Length + lowercase
      expect(result.isValid).toBe(false);
      expect(result.feedback).toContain('Add uppercase letters');
      expect(result.feedback).toContain('Add numbers');
    });

    it('should return good score for passwords with variety but short', () => {
      const result = calculatePasswordStrength('Pass1');
      expect(result.score).toBe(3); // Length missing but has lower, upper, number
      expect(result.isValid).toBe(false);
      expect(result.feedback).toContain('Password must be at least 8 characters long');
    });

    it('should return strong score for secure passwords', () => {
      const result = calculatePasswordStrength('Password123');
      expect(result.score).toBe(4);
      expect(result.isValid).toBe(true);
      expect(result.feedback).toHaveLength(0);
    });

    it('should give bonus for special characters', () => {
      const result = calculatePasswordStrength('Password123!');
      expect(result.score).toBe(4); // Capped at 4
      expect(result.isValid).toBe(true);
      expect(result.feedback).toHaveLength(0);
    });

    it('should handle empty password', () => {
      const result = calculatePasswordStrength('');
      expect(result.score).toBe(0);
      expect(result.isValid).toBe(false);
      expect(result.feedback.length).toBeGreaterThan(0);
    });

    it('should provide specific feedback for missing requirements', () => {
      const result = calculatePasswordStrength('lowercase');
      expect(result.feedback).toContain('Add uppercase letters');
      expect(result.feedback).toContain('Add numbers');
      expect(result.feedback).not.toContain('Password must be at least 8 characters long');
    });

    it('should validate minimum requirements correctly', () => {
      // Exactly meets minimum requirements
      const result = calculatePasswordStrength('Abcdef12');
      expect(result.score).toBe(4);
      expect(result.isValid).toBe(true);
      expect(result.feedback).toHaveLength(0);
    });
  });

  describe('getPasswordStrengthColor', () => {
    it('should return red for weak passwords', () => {
      expect(getPasswordStrengthColor(0)).toBe('text-red-600');
      expect(getPasswordStrengthColor(1)).toBe('text-red-600');
    });

    it('should return orange for fair passwords', () => {
      expect(getPasswordStrengthColor(2)).toBe('text-orange-500');
    });

    it('should return yellow for good passwords', () => {
      expect(getPasswordStrengthColor(3)).toBe('text-yellow-500');
    });

    it('should return green for strong passwords', () => {
      expect(getPasswordStrengthColor(4)).toBe('text-green-600');
    });

    it('should return gray for invalid scores', () => {
      expect(getPasswordStrengthColor(-1)).toBe('text-gray-400');
      expect(getPasswordStrengthColor(5)).toBe('text-gray-400');
    });
  });

  describe('getPasswordStrengthText', () => {
    it('should return appropriate text for each score', () => {
      expect(getPasswordStrengthText(0)).toBe('Weak');
      expect(getPasswordStrengthText(1)).toBe('Weak');
      expect(getPasswordStrengthText(2)).toBe('Fair');
      expect(getPasswordStrengthText(3)).toBe('Good');
      expect(getPasswordStrengthText(4)).toBe('Strong');
    });

    it('should return empty string for invalid scores', () => {
      expect(getPasswordStrengthText(-1)).toBe('');
      expect(getPasswordStrengthText(5)).toBe('');
    });
  });
});