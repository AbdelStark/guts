import type { NextConfig } from "next";

const isProd = process.env.NODE_ENV === "production";

const nextConfig: NextConfig = {
  // Enable static export for GitHub Pages
  output: "export",

  // Base path for GitHub Pages (repo name)
  // Set via NEXT_PUBLIC_BASE_PATH env var in CI
  basePath: process.env.NEXT_PUBLIC_BASE_PATH || "",

  // Asset prefix for GitHub Pages
  assetPrefix: process.env.NEXT_PUBLIC_BASE_PATH || "",

  // Use trailing slashes for cleaner static file structure
  trailingSlash: true,

  // Disable image optimization (not supported in static export)
  images: {
    unoptimized: true,
  },

  // TypeScript and ESLint during builds
  typescript: {
    // Allow builds even with type errors in CI (for faster iteration)
    // Set to false for strict type checking
    ignoreBuildErrors: false,
  },

  eslint: {
    // Allow builds even with lint warnings
    ignoreDuringBuilds: false,
  },
};

export default nextConfig;
