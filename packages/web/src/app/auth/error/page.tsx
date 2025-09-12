export default function AuthErrorPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div className="text-center">
          <h2 className="mt-6 text-3xl font-extrabold text-gray-900">
            Authentication Error
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            There was an error with authentication. Please try again.
          </p>
          <div className="mt-4">
            <a
              href="/auth/signin"
              className="font-medium text-blue-600 hover:text-blue-500"
            >
              Return to Sign In
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}