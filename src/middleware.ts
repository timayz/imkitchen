import createIntlMiddleware from 'next-intl/middleware';
import { NextRequest, NextResponse } from 'next/server';
import { getToken } from 'next-auth/jwt';
import { defaultLocale, locales, isValidLocale, type Locale } from '@/lib/i18n';

// Create next-intl middleware
const intlMiddleware = createIntlMiddleware({
  locales,
  defaultLocale,
  localePrefix: 'always',
});

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Handle API routes and static files - no locale processing needed
  if (
    pathname.startsWith('/api/') ||
    pathname.startsWith('/_next/') ||
    pathname.includes('.')
  ) {
    return handleApiAndAuth(request);
  }

  // Extract locale from pathname
  const pathnameIsMissingLocale = locales.every(
    locale => !pathname.startsWith(`/${locale}/`) && pathname !== `/${locale}`
  );

  // Redirect if there is no locale in the pathname
  if (pathnameIsMissingLocale) {
    const locale = getLocale(request);

    // Handle root path
    if (pathname === '/') {
      return NextResponse.redirect(new URL(`/${locale}`, request.url));
    }

    // Handle other paths
    return NextResponse.redirect(new URL(`/${locale}${pathname}`, request.url));
  }

  // Apply internationalization
  const intlResponse = intlMiddleware(request);
  if (intlResponse) {
    // For localized routes, also check authentication
    return handleAuthForLocalizedRoutes(request, intlResponse);
  }

  return NextResponse.next();
}

function getLocale(request: NextRequest): Locale {
  // 1. Check cookie preference
  const cookieLocale = request.cookies.get('NEXT_LOCALE')?.value;
  if (cookieLocale && isValidLocale(cookieLocale)) {
    return cookieLocale;
  }

  // 2. Check Accept-Language header
  const acceptLanguage = request.headers.get('accept-language');
  if (acceptLanguage) {
    const browserLocales = acceptLanguage
      .split(',')
      .map(lang => lang.split(';')[0].trim().slice(0, 2));

    for (const browserLocale of browserLocales) {
      if (isValidLocale(browserLocale)) {
        return browserLocale;
      }
    }
  }

  // 3. Fallback to default locale
  return defaultLocale;
}

async function handleApiAndAuth(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Allow all API routes to pass through
  if (pathname.startsWith('/api/')) {
    // For protected API routes, add user context
    const protectedApiRoutes = [
      '/api/user',
      '/api/inventory',
      '/api/recipes',
      '/api/meal-plans',
      '/api/shopping',
    ];
    const isProtectedApi = protectedApiRoutes.some(route =>
      pathname.startsWith(route)
    );

    if (isProtectedApi) {
      const token = await getToken({
        req: request,
        secret: process.env.NEXTAUTH_SECRET || '',
      });

      if (!token) {
        return new NextResponse(JSON.stringify({ error: 'Unauthorized' }), {
          status: 401,
          headers: { 'content-type': 'application/json' },
        });
      }

      // Add user context to headers
      const requestHeaders = new Headers(request.headers);
      requestHeaders.set('x-user-id', token.sub!);
      requestHeaders.set('x-household-id', token.householdId as string);

      return NextResponse.next({
        request: {
          headers: requestHeaders,
        },
      });
    }
  }

  return NextResponse.next();
}

async function handleAuthForLocalizedRoutes(
  request: NextRequest,
  intlResponse: NextResponse
) {
  const { pathname } = request.nextUrl;

  // Extract locale and path
  const locale = pathname.split('/')[1];
  const pathWithoutLocale = pathname.slice(locale.length + 1) || '/';

  // Public routes that don't require authentication
  const publicRoutes = ['/', '/login', '/register'];
  const isPublicRoute =
    publicRoutes.includes(pathWithoutLocale) ||
    pathWithoutLocale.startsWith('/login') ||
    pathWithoutLocale.startsWith('/register');

  // Allow access to public routes
  if (isPublicRoute) {
    return intlResponse;
  }

  // For protected routes, check if user has a valid token
  const token = await getToken({
    req: request,
    secret: process.env.NEXTAUTH_SECRET || '',
  });

  if (!token) {
    // Redirect to localized login page for unauthenticated users
    const loginUrl = new URL(`/${locale}/login`, request.url);
    loginUrl.searchParams.set('callbackUrl', pathname);
    return NextResponse.redirect(loginUrl);
  }

  return intlResponse;
}

export const config = {
  matcher: [
    // Match all routes except API, _next static files, and file extensions
    '/((?!api|_next/static|_next/image|favicon.ico).*)',
  ],
};
