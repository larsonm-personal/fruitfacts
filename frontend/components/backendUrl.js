function getServerBackendBase() {
  if (typeof window === 'undefined' && process.env.FRUITFACTS_SERVER_BACKEND_BASE) {
    return process.env.FRUITFACTS_SERVER_BACKEND_BASE;
  }

  const configured = process.env.NEXT_PUBLIC_BACKEND_BASE;
  if (typeof window === 'undefined' && configured === 'http://local.fruitfacts.xyz:3001') {
    return 'http://127.0.0.1:3001';
  }

  return configured;
}

module.exports = { getServerBackendBase };
