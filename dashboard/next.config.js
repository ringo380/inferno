/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  output: 'standalone',
  images: {
    domains: ['localhost'],
    unoptimized: process.env.NODE_ENV === 'production',
  },
  env: {
    INFERNO_API_URL: process.env.INFERNO_API_URL || 'http://localhost:8080',
    INFERNO_WS_URL: process.env.INFERNO_WS_URL || 'ws://localhost:8080',
  },
  async rewrites() {
    return [
      {
        source: '/api/inferno/:path*',
        destination: `${process.env.INFERNO_API_URL || 'http://localhost:8080'}/:path*`,
      },
    ];
  },
  webpack: (config, { buildId, dev, isServer, defaultLoaders, webpack }) => {
    // Optimize bundle size
    if (!dev && !isServer) {
      config.optimization.splitChunks = {
        chunks: 'all',
        cacheGroups: {
          vendor: {
            test: /[\\/]node_modules[\\/]/,
            name: 'vendors',
            priority: 10,
            reuseExistingChunk: true,
          },
          common: {
            name: 'common',
            minChunks: 2,
            priority: 5,
            reuseExistingChunk: true,
          },
        },
      };
    }
    return config;
  },
};

module.exports = nextConfig;