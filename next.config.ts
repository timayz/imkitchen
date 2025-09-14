import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  // Experimental features for better performance
  experimental: {
    // Enable optimized imports for better bundle size
    optimizePackageImports: ['@radix-ui/react-icons'],
  },
  // Add this when we implement next-intl in future stories
  // Note: App Router uses next-intl middleware instead of built-in i18n
};

export default nextConfig;
