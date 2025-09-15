import { redirect } from 'next/navigation';
import { auth } from '@/lib/auth';
import Link from 'next/link';
import type { Locale } from '@/lib/i18n';

interface SettingsPageProps {
  params: Promise<{
    locale: Locale;
  }>;
}

export default async function SettingsPage({ params }: SettingsPageProps) {
  const { locale } = await params;
  const session = await auth();

  if (!session?.user) {
    redirect(`/${locale}/login`);
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Settings</h1>
          <p className="text-gray-600 mt-2">
            Manage your account and preferences
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {/* Profile Settings Card */}
          <Link
            href={`/${locale}/settings/profile`}
            className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow"
          >
            <div className="flex items-center">
              <div className="bg-blue-100 p-3 rounded-lg">
                <svg
                  className="w-6 h-6 text-blue-600"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                  />
                </svg>
              </div>
              <div className="ml-4">
                <h3 className="text-lg font-semibold text-gray-900">Profile</h3>
                <p className="text-gray-500">
                  Personal information & preferences
                </p>
              </div>
            </div>
          </Link>

          {/* Placeholder for future settings */}
          <div className="bg-white rounded-lg shadow-md p-6 opacity-60">
            <div className="flex items-center">
              <div className="bg-gray-100 p-3 rounded-lg">
                <svg
                  className="w-6 h-6 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 17h5l-5 5v-5zM9 7h6a3 3 0 013 3v5.5L14 19H9a3 3 0 01-3-3V10a3 3 0 013-3z"
                  />
                </svg>
              </div>
              <div className="ml-4">
                <h3 className="text-lg font-semibold text-gray-400">
                  Notifications
                </h3>
                <p className="text-gray-400">Coming soon...</p>
              </div>
            </div>
          </div>

          <div className="bg-white rounded-lg shadow-md p-6 opacity-60">
            <div className="flex items-center">
              <div className="bg-gray-100 p-3 rounded-lg">
                <svg
                  className="w-6 h-6 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                  />
                </svg>
              </div>
              <div className="ml-4">
                <h3 className="text-lg font-semibold text-gray-400">Privacy</h3>
                <p className="text-gray-400">Coming soon...</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
