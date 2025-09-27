/** @type {import('next').NextConfig} */
const nextConfig = {
  // Removed deprecated appDir option (now default in Next.js 14)
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
  env: {
    INFERNO_API_URL: process.env.INFERNO_API_URL || 'tauri://localhost',
    INFERNO_WS_URL: process.env.INFERNO_WS_URL || 'tauri://localhost',
  },
  // Removed rewrites as they don't work with export mode and we're using Tauri
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