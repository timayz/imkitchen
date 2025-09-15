import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { ExpirationStatus } from '@/types/inventory';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getExpirationStatus(expirationDate: Date): ExpirationStatus {
  const now = new Date();
  const diffTime = expirationDate.getTime() - now.getTime();
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return 'expired';
  if (diffDays <= 3) return 'expiring_soon';
  if (diffDays <= 7) return 'expiring_later';
  return 'fresh';
}

export function getExpirationColor(status: ExpirationStatus): string {
  switch (status) {
    case 'expired':
      return 'text-red-600 bg-red-50';
    case 'expiring_soon':
      return 'text-red-500 bg-red-50';
    case 'expiring_later':
      return 'text-yellow-600 bg-yellow-50';
    case 'fresh':
      return 'text-green-600 bg-green-50';
    default:
      return 'text-gray-600 bg-gray-50';
  }
}

export function formatDate(date: Date): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(date);
}
