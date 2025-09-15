import { Metadata } from 'next';
import { RegisterForm } from '@/components/forms/register-form';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'Create Account | imkitchen',
  description:
    'Create your imkitchen account to start managing your kitchen with smart inventory tracking, meal planning, and recipe discovery.',
};

export default function RegisterPage() {
  return (
    <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="sm:mx-auto sm:w-full sm:max-w-md">
        <h1 className="text-center text-3xl font-bold text-gray-900">
          imkitchen
        </h1>
        <h2 className="mt-6 text-center text-2xl font-medium text-gray-900">
          Create your account
        </h2>
        <p className="mt-2 text-center text-sm text-gray-600">
          Start managing your kitchen with smart features
        </p>
      </div>

      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-lg">
        <div className="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
          <RegisterForm />
        </div>
      </div>

      <div className="mt-6 text-center">
        <p className="text-sm text-gray-600">
          Already have an account?{' '}
          <Link
            href="/login"
            className="text-orange-600 hover:text-orange-500 font-medium"
          >
            Sign in
          </Link>
        </p>
      </div>
    </div>
  );
}
