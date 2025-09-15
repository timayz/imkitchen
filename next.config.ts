import type { NextConfig } from 'next';
import createNextIntlPlugin from 'next-intl/plugin';

const withNextIntl = createNextIntlPlugin('./src/lib/i18n.ts');

const nextConfig: NextConfig = {
  outputFileTracingRoot: __dirname,
  // Experimental features for better performance
  experimental: {
    // Enable optimized imports for better bundle size
    optimizePackageImports: ['@radix-ui/react-icons'],
  },
};

export default withNextIntl(nextConfig);
