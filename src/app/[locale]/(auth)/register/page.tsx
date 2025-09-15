import { Metadata } from 'next';
import { RegisterForm } from '@/components/forms/register-form';
import Link from 'next/link';
import { getTranslations } from 'next-intl/server';

type Props = {
  params: Promise<{ locale: string }>;
};

export const metadata: Metadata = {
  title: 'Create Account | imkitchen',
  description:
    'Create your imkitchen account to start managing your kitchen with smart inventory tracking, meal planning, and recipe discovery.',
};

export default async function RegisterPage({ params }: Props) {
  await params;
  const t = await getTranslations('register');

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="sm:mx-auto sm:w-full sm:max-w-md">
        <h1 className="text-center text-3xl font-bold text-gray-900">
          imkitchen
        </h1>
        <h2 className="mt-6 text-center text-2xl font-medium text-gray-900">
          {t('title')}
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
          {t('hasAccount')}{' '}
          <Link
            href="/login"
            className="text-orange-600 hover:text-orange-500 font-medium"
          >
            {t('signIn')}
          </Link>
        </p>
      </div>
    </div>
  );
}
