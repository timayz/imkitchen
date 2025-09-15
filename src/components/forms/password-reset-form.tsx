'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  passwordResetRequestSchema,
  type PasswordResetRequestData,
} from '@/lib/validators/auth-schemas';
import Link from 'next/link';

interface PasswordResetFormProps {
  className?: string;
}

export function PasswordResetForm({ className = '' }: PasswordResetFormProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<PasswordResetRequestData>({
    resolver: zodResolver(passwordResetRequestSchema),
  });

  const onSubmit = async (data: PasswordResetRequestData) => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/auth/reset-password', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.message || 'Failed to send reset email');
      }

      setSuccess(true);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    } finally {
      setIsLoading(false);
    }
  };

  if (success) {
    return (
      <div className={`w-full max-w-md mx-auto ${className}`}>
        <div className="bg-green-50 border border-green-200 rounded-md p-6 text-center">
          <h3 className="text-lg font-medium text-green-900 mb-2">
            Check your email
          </h3>
          <p className="text-sm text-green-700">
            If an account with that email exists, you will receive a password
            reset email with instructions.
          </p>
          <div className="mt-4">
            <Link
              href="/login"
              className="text-sm text-green-600 hover:text-green-500 font-medium"
            >
              Return to login
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`w-full max-w-md mx-auto ${className}`}>
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        <div>
          <label
            htmlFor="email"
            className="block text-sm font-medium text-gray-700"
          >
            Email Address
          </label>
          <input
            {...register('email')}
            type="email"
            id="email"
            autoComplete="email"
            className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500"
            placeholder="Enter your email address"
          />
          {errors.email && (
            <p className="mt-1 text-sm text-red-600">{errors.email.message}</p>
          )}
        </div>

        {error && (
          <div className="bg-red-50 border border-red-200 rounded-md p-4">
            <p className="text-sm text-red-600">{error}</p>
          </div>
        )}

        <button
          type="submit"
          disabled={isLoading}
          className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-orange-600 hover:bg-orange-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-orange-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? 'Sending...' : 'Send Reset Email'}
        </button>

        <div className="text-center">
          <Link
            href="/login"
            className="text-sm text-gray-600 hover:text-gray-500"
          >
            Back to login
          </Link>
        </div>
      </form>
    </div>
  );
}
