import { NextIntlClientProvider } from 'next-intl';
import { getMessages } from 'next-intl/server';
import { notFound } from 'next/navigation';
import { ReactNode } from 'react';
import { SessionProvider } from '@/components/providers/session-provider';
import { isValidLocale } from '@/lib/i18n';

interface AuthLayoutProps {
  children: ReactNode;
  params: Promise<{
    locale: string;
  }>;
}

export default async function AuthLayout({
  children,
  params,
}: AuthLayoutProps) {
  const { locale } = await params;

  // Validate the locale
  if (!isValidLocale(locale)) {
    notFound();
  }

  // Get the messages for the locale
  const messages = await getMessages({ locale });

  return (
    <NextIntlClientProvider messages={messages}>
      <SessionProvider>{children}</SessionProvider>
    </NextIntlClientProvider>
  );
}
