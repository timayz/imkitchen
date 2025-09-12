'use client';

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { updateProfileSchema, type UpdateProfileInput } from '@imkitchen/shared';
import { trpc } from '../../lib/trpc';

export function ProfileUpdateForm() {
  const [submitMessage, setSubmitMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const { data: userProfile, isLoading } = trpc.auth.me.useQuery();

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<UpdateProfileInput>({
    resolver: zodResolver(updateProfileSchema),
  });

  const updateProfileMutation = trpc.auth.updateProfile.useMutation({
    onSuccess: (data) => {
      setSubmitMessage({ type: 'success', text: data.message });
      // Reset form to remove dirty state
      const resetData: UpdateProfileInput = {
        name: data.user.name || '',
        preferences: {
          dietaryRestrictions: [],
          cookingSkillLevel: undefined,
          preferredCuisines: [],
        }
      };
      reset(resetData);
    },
    onError: (error) => {
      setSubmitMessage({ type: 'error', text: error.message });
    },
  });

  // Reset form when user data is loaded
  useEffect(() => {
    if (userProfile) {
      // Use any to avoid TypeScript recursion issues with complex Prisma types
      reset({
        name: userProfile.name || '',
        preferences: {
          dietaryRestrictions: [],
          cookingSkillLevel: undefined,
          preferredCuisines: [],
        }
      } as any);
    }
  }, [userProfile]);

  const onSubmit = async (data: UpdateProfileInput) => {
    setSubmitMessage(null);
    updateProfileMutation.mutate(data);
  };

  if (isLoading) {
    return (
      <div className="text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p className="text-gray-600">Loading profile...</p>
      </div>
    );
  }

  return (
    <div className="max-w-2xl">
      <h2 className="text-lg font-medium text-gray-900 mb-6">Profile Information</h2>
      
      {submitMessage && (
        <div className={`mb-4 p-4 rounded-md ${
          submitMessage.type === 'success' 
            ? 'bg-green-50 border border-green-200 text-green-800' 
            : 'bg-red-50 border border-red-200 text-red-800'
        }`}>
          {submitMessage.text}
        </div>
      )}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Basic Information */}
        <div className="space-y-4">
          <h3 className="text-md font-medium text-gray-900">Basic Information</h3>
          
          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
              Full Name
            </label>
            <input
              type="text"
              id="name"
              {...register('name')}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Enter your full name"
            />
            {errors.name && (
              <p className="mt-1 text-sm text-red-600">{errors.name.message}</p>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Email Address
            </label>
            <input
              type="email"
              value={userProfile?.email || ''}
              disabled
              className="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500 cursor-not-allowed"
            />
            <p className="mt-1 text-xs text-gray-500">Email address cannot be changed</p>
          </div>
        </div>

        {/* Cooking Preferences */}
        <div className="space-y-4">
          <h3 className="text-md font-medium text-gray-900">Cooking Preferences</h3>
          
          <div>
            <label htmlFor="cookingSkillLevel" className="block text-sm font-medium text-gray-700 mb-1">
              Cooking Skill Level
            </label>
            <select
              id="cookingSkillLevel"
              {...register('preferences.cookingSkillLevel')}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="">Select skill level</option>
              <option value="beginner">Beginner</option>
              <option value="intermediate">Intermediate</option>
              <option value="advanced">Advanced</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Dietary Restrictions
            </label>
            <div className="space-y-2">
              {['Vegetarian', 'Vegan', 'Gluten-Free', 'Dairy-Free', 'Nut-Free', 'Low-Carb', 'Keto'].map((restriction) => (
                <label key={restriction} className="flex items-center">
                  <input
                    type="checkbox"
                    value={restriction}
                    {...register('preferences.dietaryRestrictions')}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700">{restriction}</span>
                </label>
              ))}
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Preferred Cuisines
            </label>
            <div className="grid grid-cols-2 gap-2">
              {['Italian', 'Mexican', 'Asian', 'Mediterranean', 'American', 'French', 'Indian', 'Thai'].map((cuisine) => (
                <label key={cuisine} className="flex items-center">
                  <input
                    type="checkbox"
                    value={cuisine}
                    {...register('preferences.preferredCuisines')}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700">{cuisine}</span>
                </label>
              ))}
            </div>
          </div>
        </div>

        {/* Submit Button */}
        <div className="pt-4">
          <button
            type="submit"
            disabled={!isDirty || updateProfileMutation.isLoading}
            className="bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {updateProfileMutation.isLoading ? 'Saving...' : 'Save Changes'}
          </button>
          {!isDirty && (
            <p className="mt-2 text-sm text-gray-500">No changes to save</p>
          )}
        </div>
      </form>
    </div>
  );
}