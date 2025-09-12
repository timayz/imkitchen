export interface PasswordStrength {
  score: number; // 0-4
  feedback: string[];
  isValid: boolean;
}

export function calculatePasswordStrength(password: string): PasswordStrength {
  const feedback: string[] = [];
  let score = 0;

  // Length check
  if (password.length >= 8) {
    score += 1;
  } else {
    feedback.push("Password must be at least 8 characters long");
  }

  // Lowercase check
  if (/[a-z]/.test(password)) {
    score += 1;
  } else {
    feedback.push("Add lowercase letters");
  }

  // Uppercase check
  if (/[A-Z]/.test(password)) {
    score += 1;
  } else {
    feedback.push("Add uppercase letters");
  }

  // Number check
  if (/\d/.test(password)) {
    score += 1;
  } else {
    feedback.push("Add numbers");
  }

  // Special character check (bonus)
  if (/[!@#$%^&*(),.?":{}|<>]/.test(password)) {
    score += 1;
  }

  const isValid = score >= 4 && password.length >= 8;

  return {
    score: Math.min(score, 4),
    feedback,
    isValid,
  };
}

export function getPasswordStrengthColor(score: number): string {
  switch (score) {
    case 0:
    case 1:
      return "text-red-600";
    case 2:
      return "text-orange-500";
    case 3:
      return "text-yellow-500";
    case 4:
      return "text-green-600";
    default:
      return "text-gray-400";
  }
}

export function getPasswordStrengthText(score: number): string {
  switch (score) {
    case 0:
    case 1:
      return "Weak";
    case 2:
      return "Fair";
    case 3:
      return "Good";
    case 4:
      return "Strong";
    default:
      return "";
  }
}