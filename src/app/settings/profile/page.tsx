import { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '@/lib/auth';
import { AuthService } from '@/lib/services/auth-service';
import { UserProfileForm } from '@/components/forms/user-profile-form';

export const metadata: Metadata = {
  title: 'Profile Settings | imkitchen',
  description:
    'Manage your personal information, dietary preferences, and account settings.',
};

export default async function ProfilePage() {
  const session = await getServerSession(authOptions);

  if (!session?.user) {
    redirect('/login');
  }

  const userProfile = await AuthService.getUserProfile(session.user.id);

  if (!userProfile) {
    redirect('/login');
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <div className="bg-white shadow sm:rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <div className="mb-6">
            <h1 className="text-2xl font-bold text-gray-900">
              Profile Settings
            </h1>
            <p className="mt-1 text-sm text-gray-600">
              Manage your personal information and preferences
            </p>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div>
              <h2 className="text-lg font-medium text-gray-900 mb-4">
                Personal Information
              </h2>
              <UserProfileForm user={userProfile} />
            </div>

            <div>
              <h2 className="text-lg font-medium text-gray-900 mb-4">
                Account Details
              </h2>
              <div className="space-y-4">
                <div>
                  <dt className="text-sm font-medium text-gray-500">Email</dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {userProfile.email}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">
                    Household
                  </dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {userProfile.household.name} (
                    {userProfile.household.memberCount} members)
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">
                    Member Since
                  </dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {new Date(userProfile.createdAt).toLocaleDateString()}
                  </dd>
                </div>
              </div>

              <div className="mt-8">
                <h3 className="text-lg font-medium text-gray-900 mb-4">
                  Account Actions
                </h3>
                <div className="space-y-3">
                  <button
                    type="button"
                    className="block w-full text-left px-4 py-2 text-sm text-orange-600 hover:bg-orange-50 rounded-md border border-orange-200"
                  >
                    Change Password
                  </button>
                  <button
                    type="button"
                    className="block w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50 rounded-md border border-red-200"
                  >
                    Sign Out
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
