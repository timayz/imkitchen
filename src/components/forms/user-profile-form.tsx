'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  updateProfileSchema,
  type UpdateProfileData,
} from '@/lib/validators/auth-schemas';
import { UserProfile } from '@/types/auth';
import { DietaryPreference, Language } from '@prisma/client';
import { signOut } from 'next-auth/react';

interface UserProfileFormProps {
  user: UserProfile;
}

export function UserProfileForm({ user }: UserProfileFormProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const router = useRouter();

  const {
    register,
    handleSubmit,
    formState: { errors, isDirty },
  } = useForm<UpdateProfileData>({
    resolver: zodResolver(updateProfileSchema),
    defaultValues: {
      name: user.name,
      dietaryPreferences: user.dietaryPreferences,
      allergies: user.allergies,
      language: user.language,
      timezone: user.timezone,
    },
  });

  const onSubmit = async (data: UpdateProfileData) => {
    setIsLoading(true);
    setError(null);
    setSuccess(false);

    try {
      const response = await fetch('/api/user/profile', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.message || 'Update failed');
      }

      setSuccess(true);
      router.refresh();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignOut = async () => {
    await signOut({ callbackUrl: '/login' });
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
      <div>
        <label
          htmlFor="name"
          className="block text-sm font-medium text-gray-700"
        >
          Full Name
        </label>
        <input
          {...register('name')}
          type="text"
          id="name"
          className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500"
        />
        {errors.name && (
          <p className="mt-1 text-sm text-red-600">{errors.name.message}</p>
        )}
      </div>

      <div>
        <span className="block text-sm font-medium text-gray-700 mb-2">
          Dietary Preferences
        </span>
        <div className="space-y-2">
          {Object.values(DietaryPreference).map(preference => (
            <label key={preference} className="flex items-center">
              <input
                {...register('dietaryPreferences')}
                type="checkbox"
                value={preference}
                className="h-4 w-4 text-orange-600 focus:ring-orange-500 border-gray-300 rounded"
              />
              <span className="ml-2 text-sm text-gray-700 capitalize">
                {preference.replace('_', ' ')}
              </span>
            </label>
          ))}
        </div>
      </div>

      <div>
        <label
          htmlFor="allergies"
          className="block text-sm font-medium text-gray-700"
        >
          Food Allergies
        </label>
        <textarea
          {...register('allergies')}
          id="allergies"
          rows={3}
          className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500"
          placeholder="Enter any food allergies, separated by commas"
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label
            htmlFor="language"
            className="block text-sm font-medium text-gray-700"
          >
            Language
          </label>
          <select
            {...register('language')}
            id="language"
            className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500"
          >
            <option value={Language.EN}>English</option>
            <option value={Language.ES}>Spanish</option>
            <option value={Language.FR}>French</option>
            <option value={Language.DE}>German</option>
          </select>
        </div>

        <div>
          <label
            htmlFor="timezone"
            className="block text-sm font-medium text-gray-700"
          >
            Timezone
          </label>
          <select
            {...register('timezone')}
            id="timezone"
            className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500"
          >
            <option value="UTC">UTC</option>
            <option value="America/New_York">Eastern Time</option>
            <option value="America/Chicago">Central Time</option>
            <option value="America/Denver">Mountain Time</option>
            <option value="America/Los_Angeles">Pacific Time</option>
            <option value="Europe/London">London</option>
            <option value="Europe/Paris">Paris</option>
            <option value="Europe/Berlin">Berlin</option>
          </select>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <p className="text-sm text-red-600">{error}</p>
        </div>
      )}

      {success && (
        <div className="bg-green-50 border border-green-200 rounded-md p-4">
          <p className="text-sm text-green-600">
            Profile updated successfully!
          </p>
        </div>
      )}

      <div className="flex space-x-4">
        <button
          type="submit"
          disabled={isLoading || !isDirty}
          className="flex-1 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-orange-600 hover:bg-orange-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-orange-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? 'Saving...' : 'Save Changes'}
        </button>

        <button
          type="button"
          onClick={handleSignOut}
          className="py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-orange-500"
        >
          Sign Out
        </button>
      </div>
    </form>
  );
}
