const fs = require('fs');
const vm = require('vm');
const path = require('path');
const check = fs.readFileSync(path.join(__dirname, '../prebuild.js'), 'utf8');

function checkVersion(node) {
  vm.runInNewContext(check, {
    process: { versions: { node }, exit: () => { throw new Error('unsupported Node'); } },
    console: { log() {}, error() {} }
  });
}

test.each(['22.23.4', '24.22.0', '24.21.0', '26.12.1'])('rejects unselected/unpatched Node %s', (version) => {
  expect(() => checkVersion(version)).toThrow('unsupported Node');
});

test.each(['24.22.1', '24.22.2', '24.23.0'])('accepts patched Node %s', (version) => {
  expect(() => checkVersion(version)).not.toThrow();
});
