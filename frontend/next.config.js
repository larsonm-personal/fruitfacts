module.exports = {
  webpack5: true,
  webpack: (config) => {
    config.resolve.fallback = { fs: false, path: false }; // fixes "Module not found: Can't resolve 'fs'" see https://stackoverflow.com/questions/64926174/module-not-found-cant-resolve-fs-in-next-js-application
    return config;
  }
};
