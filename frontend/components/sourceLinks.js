// Source links describe this project, independently of server-specific .env overrides.
export const repositoryUrl = 'https://github.com/larsonm-personal/fruitfacts';
export function sourceUrl(path = '', revision = 'main') {
  return `${repositoryUrl}/tree/${revision}/${path.replace(/^\/+/, '')}`;
}
export function commitUrl(hash) {
  return `${repositoryUrl}/tree/${hash}`;
}
