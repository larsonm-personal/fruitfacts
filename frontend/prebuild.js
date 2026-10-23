// Node's bundled Brotli fix for CVE-2026-111497 first shipped in 22.23.4.
// npm runs this before builds AND starts, including the existing systemd service.
const [major, minor, patch] = process.versions.node.split('.').map(Number);
if (major !== 22 || minor < 23 || (minor === 23 && patch < 4)) {
  console.error(`Node ${process.versions.node} is unsupported. Use Node >=22.23.4 <23 (see .nvmrc).`);
  process.exit(1);
}
console.log(`Using patched Node ${process.versions.node}`);
