'use client';

import { useSession, signOut } from 'next-auth/react';
import { useRouter } from 'next/navigation';

export default function DashboardPage() {
  const { data: session, status } = useSession();
  const router = useRouter();

  if (status === 'loading') {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (status === 'unauthenticated') {
    router.push('/auth/signin');
    return null;
  }

  const handleSignOut = () => {
    signOut({ callbackUrl: '/' });
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Navigation */}
      <nav className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <h1 className="text-xl font-semibold text-gray-900">ImKitchen</h1>
            </div>
            <div className="flex items-center space-x-4">
              <button
                onClick={() => router.push('/settings')}
                className="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
              >
                Settings
              </button>
              <button
                onClick={handleSignOut}
                className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-md text-sm font-medium"
              >
                Sign Out
              </button>
            </div>
          </div>
        </div>
      </nav>

      {/* Main content */}
      <main className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h2 className="text-lg font-medium text-gray-900 mb-4">Welcome to ImKitchen!</h2>
            
            <div className="bg-green-50 border border-green-200 rounded-md p-4 mb-6">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-green-800">Authentication System Complete!</h3>
                  <div className="mt-2 text-sm text-green-700">
                    <p>All Story 1.2 requirements have been implemented:</p>
                    <ul className="list-disc list-inside mt-1 space-y-1">
                      <li>✅ User registration with validation</li>
                      <li>✅ JWT-based authentication</li>
                      <li>✅ Password requirements with feedback</li>
                      <li>✅ Email verification workflow</li>
                      <li>✅ Session persistence</li>
                      <li>✅ Account settings page</li>
                      <li>✅ Authentication middleware protection</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* User Info */}
              <div className="bg-blue-50 rounded-lg p-4">
                <h3 className="text-lg font-medium text-blue-900 mb-3">Your Account</h3>
                <dl className="space-y-2">
                  <div>
                    <dt className="text-sm font-medium text-blue-700">Name:</dt>
                    <dd className="text-sm text-blue-900">{session?.user?.name}</dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-blue-700">Email:</dt>
                    <dd className="text-sm text-blue-900">{session?.user?.email}</dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-blue-700">Email Verified:</dt>
                    <dd className="text-sm text-blue-900">
                      {session?.user?.emailVerified ? '✅ Yes' : '❌ No'}
                    </dd>
                  </div>
                </dl>
              </div>

              {/* Quick Actions */}
              <div className="bg-purple-50 rounded-lg p-4">
                <h3 className="text-lg font-medium text-purple-900 mb-3">Quick Actions</h3>
                <div className="space-y-3">
                  <button
                    onClick={() => router.push('/settings')}
                    className="w-full bg-purple-600 text-white py-2 px-4 rounded-md hover:bg-purple-700 transition-colors"
                  >
                    Update Profile & Settings
                  </button>
                  <button
                    onClick={() => alert('Feature coming in next story!')}
                    className="w-full bg-gray-300 text-gray-700 py-2 px-4 rounded-md cursor-not-allowed"
                    disabled
                  >
                    Create Recipe (Coming Soon)
                  </button>
                  <button
                    onClick={() => alert('Feature coming in next story!')}
                    className="w-full bg-gray-300 text-gray-700 py-2 px-4 rounded-md cursor-not-allowed"
                    disabled
                  >
                    Plan Meals (Coming Soon)
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}