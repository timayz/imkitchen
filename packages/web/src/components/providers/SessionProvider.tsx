'use client';

import { SessionProvider as NextAuthSessionProvider } from 'next-auth/react';
import { ReactNode } from 'react';
import { TRPCProvider } from './TRPCProvider';

interface SessionProviderProps {
  children: ReactNode;
}

export function SessionProvider({ children }: SessionProviderProps) {
  return (
    <NextAuthSessionProvider>
      <TRPCProvider>
        {children}
      </TRPCProvider>
    </NextAuthSessionProvider>
  );
}