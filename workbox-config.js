// Workbox Configuration - Story 5.2
// Generates service worker with precaching and runtime caching strategies

module.exports = {
  // Use injectManifest mode with custom service worker source
  globDirectory: 'static/',
  globPatterns: [
    '**/*.{css,js,png,svg,ico,woff2,woff,ttf,eot}'
  ],
  swDest: 'static/sw.js',
  swSrc: 'static/js/sw-source.js',

  // Prepend /static/ to all URLs since files are served under /static prefix
  modifyURLPrefix: {
    '': '/static/'
  },

  // Maximum cache sizes
  maximumFileSizeToCacheInBytes: 5 * 1024 * 1024, // 5MB
};
