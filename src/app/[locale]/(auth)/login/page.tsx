import { Suspense } from 'react';
import { Metadata } from 'next';
import { LoginForm } from '@/components/forms/login-form';
import { getTranslations } from 'next-intl/server';

type Props = {
  params: Promise<{ locale: string }>;
};

export const metadata: Metadata = {
  title: 'Sign In | imkitchen',
  description:
    'Sign in to your imkitchen account to access your personalized kitchen management features.',
};

function LoginFormWrapper() {
  return <LoginForm />;
}

export default async function LoginPage({ params }: Props) {
  const { locale } = await params;
  const t = await getTranslations({ locale, namespace: 'login' });

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
          Access your personalized kitchen management features
        </p>
      </div>

      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
          <Suspense fallback={<div>Loading...</div>}>
            <LoginFormWrapper />
          </Suspense>
        </div>
      </div>
    </div>
  );
}
