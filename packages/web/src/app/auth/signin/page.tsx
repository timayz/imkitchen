'use client';

import { useState } from 'react';
import { LoginForm } from '../../../components/auth/LoginForm';
import { RegisterForm } from '../../../components/auth/RegisterForm';

export default function SignInPage() {
  const [mode, setMode] = useState<'login' | 'register'>('login');

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        {mode === 'login' ? (
          <LoginForm onSwitchToRegister={() => setMode('register')} />
        ) : (
          <RegisterForm 
            onSuccess={() => setMode('login')}
            onSwitchToLogin={() => setMode('login')} 
          />
        )}
      </div>
    </div>
  );
}