'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { changePasswordSchema, type ChangePasswordInput } from '@imkitchen/shared';
import { trpc } from '../../lib/trpc';
import { calculatePasswordStrength, getPasswordStrengthColor, getPasswordStrengthText } from '../../lib/password';

export function PasswordChangeForm() {
  const [submitMessage, setSubmitMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<ChangePasswordInput>({
    resolver: zodResolver(changePasswordSchema),
  });

  const watchNewPassword = watch('newPassword', '');
  const passwordStrength = calculatePasswordStrength(watchNewPassword);

  const changePasswordMutation = trpc.auth.changePassword.useMutation({
    onSuccess: (data) => {
      setSubmitMessage({ type: 'success', text: data.message });
      reset(); // Clear the form
    },
    onError: (error) => {
      setSubmitMessage({ type: 'error', text: error.message });
    },
  });

  const onSubmit = async (data: ChangePasswordInput) => {
    setSubmitMessage(null);
    changePasswordMutation.mutate(data);
  };

  return (
    <div className="max-w-2xl">
      <h2 className="text-lg font-medium text-gray-900 mb-6">Password & Security</h2>
      
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
        <div className="space-y-4">
          <h3 className="text-md font-medium text-gray-900">Change Password</h3>
          
          {/* Current Password */}
          <div>
            <label htmlFor="currentPassword" className="block text-sm font-medium text-gray-700 mb-1">
              Current Password
            </label>
            <input
              type="password"
              id="currentPassword"
              {...register('currentPassword')}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Enter your current password"
            />
            {errors.currentPassword && (
              <p className="mt-1 text-sm text-red-600">{errors.currentPassword.message}</p>
            )}
          </div>

          {/* New Password */}
          <div>
            <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700 mb-1">
              New Password
            </label>
            <input
              type="password"
              id="newPassword"
              {...register('newPassword')}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Enter your new password"
            />
            {errors.newPassword && (
              <p className="mt-1 text-sm text-red-600">{errors.newPassword.message}</p>
            )}
            
            {/* Password Strength Indicator */}
            {watchNewPassword && (
              <div className="mt-2">
                <div className="flex items-center space-x-2">
                  <div className="flex-1 bg-gray-200 rounded-full h-2">
                    <div
                      className={`h-2 rounded-full transition-all duration-300 ${
                        passwordStrength.score === 0 ? 'bg-gray-200' :
                        passwordStrength.score === 1 ? 'bg-red-500' :
                        passwordStrength.score === 2 ? 'bg-orange-500' :
                        passwordStrength.score === 3 ? 'bg-yellow-500' :
                        'bg-green-500'
                      }`}
                      style={{ width: `${(passwordStrength.score / 4) * 100}%` }}
                    />
                  </div>
                  <span className={`text-sm font-medium ${getPasswordStrengthColor(passwordStrength.score)}`}>
                    {getPasswordStrengthText(passwordStrength.score)}
                  </span>
                </div>
                {passwordStrength.feedback.length > 0 && (
                  <ul className="mt-1 text-xs text-gray-600">
                    {passwordStrength.feedback.map((feedback, index) => (
                      <li key={index}>• {feedback}</li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </div>

          {/* Confirm New Password */}
          <div>
            <label htmlFor="confirmNewPassword" className="block text-sm font-medium text-gray-700 mb-1">
              Confirm New Password
            </label>
            <input
              type="password"
              id="confirmNewPassword"
              {...register('confirmNewPassword')}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Confirm your new password"
            />
            {errors.confirmNewPassword && (
              <p className="mt-1 text-sm text-red-600">{errors.confirmNewPassword.message}</p>
            )}
          </div>
        </div>

        {/* Submit Button */}
        <div className="pt-4">
          <button
            type="submit"
            disabled={changePasswordMutation.isLoading}
            className="bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-blue-400 disabled:cursor-not-allowed transition-colors"
          >
            {changePasswordMutation.isLoading ? 'Changing Password...' : 'Change Password'}
          </button>
        </div>

        {/* Security Tips */}
        <div className="mt-6 p-4 bg-blue-50 rounded-md">
          <h4 className="text-sm font-medium text-blue-900 mb-2">Password Security Tips</h4>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• Use a unique password for your ImKitchen account</li>
            <li>• Include uppercase letters, lowercase letters, and numbers</li>
            <li>• Make your password at least 8 characters long</li>
            <li>• Consider using a password manager</li>
          </ul>
        </div>
      </form>
    </div>
  );
}