// Adapted from the 2026 branch's check_local_routes.ps1 route enumeration
// Check an already-running local build; do not install, build, or stop services
const fs = require('node:fs');
const path = require('node:path');

const base = new URL(process.argv[2] || 'http://127.0.0.1:3030');
if (!['127.0.0.1', 'localhost', '[::1]'].includes(base.hostname)) {
  throw new Error('Use a loopback URL for this local route check');
}
const limit = Number(process.argv[3] || 0);
if (!Number.isSafeInteger(limit) || limit < 0)
  throw new Error('Limit must be a nonnegative integer');
const references = path.join(__dirname, '../plant_database/references');
const routes = new Set([
  '/',
  '/plants',
  '/dirs',
  '/search?searchType=base&page=1&perPage=50&orderBy=name_then_type&order=asc'
]);
const routePath = (value) =>
  value
    .split('/')
    .map((part) => encodeURIComponent(part.replaceAll(' ', '_')))
    .join('/');
function visit(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) visit(full);
    else if (entry.name.endsWith('.json5')) {
      const relative = path.relative(references, full).replaceAll(path.sep, '/').slice(0, -6);
      routes.add(`/collections/${routePath(relative)}`);
      const parts = relative.split('/');
      for (let end = 1; end < parts.length; end++)
        routes.add(`/dirs/${routePath(parts.slice(0, end).join('/'))}`);
    }
  }
}
visit(references);
(async () => {
  let checked = 0;
  let failed = 0;
  for (const route of routes) {
    if (limit && checked >= limit) break;
    checked++;
    try {
      const response = await fetch(new URL(route, base), {
        redirect: 'manual',
        signal: AbortSignal.timeout(30000)
      });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const html = await response.text();
      const nextData = html.match(/<script id="__NEXT_DATA__"[^>]*>([\s\S]*?)<\/script>/);
      if (!nextData) throw new Error('Missing Next page data');
      const error = JSON.parse(nextData[1]).props?.pageProps?.errorMessage;
      if (error) throw new Error(error);
    } catch (error) {
      failed++;
      console.error(`${route}: ${error.message}`);
    }
  }
  console.log(`Checked ${checked} routes; ${failed} failed`);
  process.exitCode = failed ? 1 : 0;
})().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
