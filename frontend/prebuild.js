// Node 24.22.1 is the patched Active LTS baseline for builds and systemd starts.
const [major, minor, patch] = process.versions.node.split('.').map(Number);
if (major !== 24 || minor < 22 || (minor === 22 && patch < 1)) {
  console.error(`Node ${process.versions.node} is unsupported. Use Node >=24.22.1 <25 (see .nvmrc).`);
  process.exit(1);
}
console.log(`Using patched Node ${process.versions.node}`);
