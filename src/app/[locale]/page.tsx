import { redirect } from 'next/navigation';
import { auth } from '@/lib/auth';
import type { Locale } from '@/lib/i18n';

interface HomePageProps {
  params: Promise<{
    locale: Locale;
  }>;
}

export default async function HomePage({ params }: HomePageProps) {
  const { locale } = await params;
  const session = await auth();

  // Redirect authenticated users to dashboard
  if (session?.user) {
    redirect(`/${locale}/dashboard`);
  }

  // Redirect unauthenticated users to login
  redirect(`/${locale}/login`);
}
