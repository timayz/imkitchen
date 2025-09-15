import * as Sentry from '@sentry/nextjs';
import { env, isProduction, isDevelopment } from '@/lib/config';

/**
 * Initialize Sentry for error tracking and performance monitoring
 * Only initialized if SENTRY_DSN is provided
 */
export function initSentry() {
  if (!process.env.SENTRY_DSN) {
    if (isDevelopment) {
      console.log('Sentry DSN not provided - error tracking disabled');
    }
    return;
  }

  Sentry.init({
    dsn: process.env.SENTRY_DSN,

    // Environment configuration
    environment: env.NODE_ENV,

    // Performance monitoring
    tracesSampleRate: isProduction ? 0.1 : 1.0, // Sample 10% in production, 100% in dev

    // Session replay for debugging (only in production with user consent)
    replaysSessionSampleRate: 0.0, // Disabled by default
    replaysOnErrorSampleRate: isProduction ? 1.0 : 0.0, // Capture replays on errors in production

    // Filter out sensitive information
    beforeSend(event) {
      // Filter out development errors
      if (isDevelopment && event.environment === 'development') {
        return null;
      }

      // Remove sensitive data from error context
      if (event.request) {
        // Remove authentication headers
        if (event.request.headers) {
          delete event.request.headers['authorization'];
          delete event.request.headers['cookie'];
        }

        // Remove sensitive query parameters
        if (event.request.query_string) {
          const queryString =
            typeof event.request.query_string === 'string'
              ? event.request.query_string
              : // eslint-disable-next-line @typescript-eslint/no-explicit-any
                new URLSearchParams(
                  event.request.query_string as any
                ).toString();
          event.request.query_string = queryString.replace(
            /(\?|&)(password|token|secret|key)=[^&]*/gi,
            '$1$2=[REDACTED]'
          );
        }
      }

      // Filter out certain error types
      if (event.exception?.values?.[0]?.type === 'ChunkLoadError') {
        return null; // Ignore chunk load errors (usually network issues)
      }

      return event;
    },

    // Integrations
    integrations: [
      Sentry.extraErrorDataIntegration(),
      Sentry.replayIntegration({
        maskAllText: true,
        blockAllMedia: true,
      }),
    ],

    // Release information
    release:
      process.env.VERCEL_GIT_COMMIT_SHA ||
      process.env.npm_package_version ||
      'unknown',

    // Additional tags
    initialScope: {
      tags: {
        component: 'imkitchen-app',
        deployment: process.env.VERCEL_ENV || 'local',
      },
    },
  });
}

/**
 * Capture an exception with additional context
 */
export function captureException(
  error: Error,
  context?: Record<string, unknown>
) {
  Sentry.captureException(error, {
    tags: {
      component: 'imkitchen-app',
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      ...((context?.tags as Record<string, any>) || {}),
    },
    extra: context || {},
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any);
}

/**
 * Capture a message with severity level
 */
export function captureMessage(
  message: string,
  level: 'fatal' | 'error' | 'warning' | 'log' | 'info' | 'debug' = 'info',
  context?: Record<string, unknown>
) {
  Sentry.captureMessage(message, level);
  if (context) {
    Sentry.withScope(scope => {
      scope.setTags({
        component: 'imkitchen-app',
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        ...((context.tags as Record<string, any>) || {}),
      });
      scope.setExtras(context);
    });
  }
}

/**
 * Add user context to Sentry scope
 */
export function setUserContext(user: {
  id: string;
  email?: string;
  username?: string;
}) {
  Sentry.setUser({
    id: user.id,
    ...(user.email ? { email: user.email } : {}),
    ...(user.username ? { username: user.username } : {}),
  });
}

/**
 * Add breadcrumb for debugging
 */
export function addBreadcrumb(
  message: string,
  category: string = 'custom',
  level: 'fatal' | 'error' | 'warning' | 'log' | 'info' | 'debug' = 'info',
  data?: Record<string, unknown>
) {
  Sentry.addBreadcrumb({
    message,
    category,
    level,
    data: data || {},
    timestamp: Date.now() / 1000,
  });
}

/**
 * Profile a function execution with Sentry tracing
 */
export async function profileExecution<T>(
  name: string,
  operation: () => Promise<T> | T,
  context?: Record<string, unknown>
): Promise<T> {
  return Sentry.startSpan(
    {
      name,
      op: 'function',
      attributes: {
        'imkitchen.component': 'backend',
        ...context,
      },
    },
    operation
  );
}

/**
 * Flush Sentry events (useful for serverless environments)
 */
export async function flushSentry(timeout: number = 2000): Promise<boolean> {
  return Sentry.flush(timeout);
}

/**
 * Check if Sentry is enabled
 */
export function isSentryEnabled(): boolean {
  return !!process.env.SENTRY_DSN;
}
