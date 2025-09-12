/** @type {import('next').NextConfig} */
const nextConfig = {
  typescript: {
    ignoreBuildErrors: false,
  },
  eslint: {
    ignoreDuringBuilds: false,
  },
  experimental: {
    // Improve webpack caching reliability
    webpackBuildWorker: true,
  },
  webpack: (config, { dev }) => {
    if (dev) {
      // Disable webpack cache in development to prevent corruption
      config.cache = false;
    }
    return config;
  },
}

module.exports = nextConfig