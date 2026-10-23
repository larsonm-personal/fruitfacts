"""Isolated deployment control-flow tests; npm/cargo/systemd/HTTP are stubbed."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('deploy.sh')

class DeployTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.source = self.base / 'source'
        self.state = self.base / 'state'
        self.root = self.base / 'releases'
        self.bin = self.base / 'bin'
        for path in (self.source, self.state, self.root, self.bin):
            path.mkdir()
        (self.source / 'scripts').mkdir()
        shutil.copy2(SCRIPT, self.source / 'scripts/deploy.sh')
        for name in ('backend/.env', 'backend/.env.release', 'frontend/.env',
                     'frontend/.env.production', 'backend/database.sqlite3',
                     'frontend/public/data/photo.png',
                     'backend/src/gazetteer_load/2021_Gaz_zcta_national.txt'):
            p = self.state / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text('GEOID\n' if name.endswith('.txt') else 'preserve-me\n')
        for name in ('backend/src/gazetteer_load/file', 'frontend/public/data/manifest.dvc'):
            p = self.source / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text('tracked\n')
        for args in (['init', '-b', 'main'], ['add', '.'],
                     ['-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'fixture']):
            subprocess.run(['git', '-C', str(self.source), *args], check=True, capture_output=True)
        self.root.joinpath('current').symlink_to(self.state)
        mock = '''#!/usr/bin/env bash
set -eu
name=${0##*/}
echo "$name $*" >> "$TEST_LOG"
case $name in
node) [[ ${FAIL_NODE:-0} != 1 ]] ;;
npm) [[ ${FAIL_BUILD:-0} != 1 ]] ;;
cargo) mkdir -p target/release; printf binary > target/release/harvest-chart-server ;;
sudo) exec "$@" ;;
systemctl)
 if [[ $1 == show ]]; then
  case "$*" in
   *ExecStart*frontend*) echo "$(command -v node) $TEST_ROOT/current/frontend/node_modules/next/dist/bin/next" ;;
   *ExecStart*) echo "$TEST_ROOT/current/backend/harvest-chart-server" ;;
   *frontend*) echo "$TEST_ROOT/current/frontend" ;;
   *) echo "$TEST_STATE/backend" ;;
  esac
 fi ;;
sqlite3)
 if [[ $* == *quick_check* ]]; then echo ok; else cp "$1" database-before-activation.sqlite3; fi ;;
curl) [[ ${FAIL_HEALTH:-0} != 1 ]] ;;
sleep) : ;;
esac
'''
        for name in ('node', 'npm', 'cargo', 'sudo', 'systemctl', 'sqlite3', 'curl', 'sleep'):
            p = self.bin / name
            p.write_text(mock)
            p.chmod(0o755)
        self.env = dict(os.environ, PATH=f'{self.bin}:{os.environ["PATH"]}',
                        TEST_ROOT=str(self.root), TEST_STATE=str(self.state),
                        TEST_LOG=str(self.base / 'commands'))
        self.before = {str(p): p.read_bytes() for p in self.state.rglob('*') if p.is_file()}

    def run_deploy(self, *args, **env):
        return subprocess.run(['bash', str(self.source / 'scripts/deploy.sh'),
                               str(self.state), str(self.root), *args],
                              env=dict(self.env, **env), text=True, capture_output=True)

    def assert_preserved(self):
        self.assertEqual(self.before, {str(p): p.read_bytes() for p in self.state.rglob('*') if p.is_file()})

    def test_repeated_activation_preserves_state(self):
        for _ in range(2):
            result = self.run_deploy('--activate')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assert_preserved()
        self.assertEqual(len(list(self.root.glob('release-*'))), 2)
        self.assertEqual((self.root / 'current/frontend/public/data/photo.png').read_text(), 'preserve-me\n')

    def test_build_only_does_not_switch(self):
        result = self.run_deploy()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / 'current').resolve(), self.state)
        self.assertNotIn('sudo', (self.base / 'commands').read_text())
        self.assert_preserved()

    def test_failed_build_does_not_switch(self):
        self.assertNotEqual(self.run_deploy('--activate', FAIL_BUILD='1').returncode, 0)
        self.assertEqual((self.root / 'current').resolve(), self.state)
        self.assert_preserved()

    def test_failed_health_rolls_back(self):
        self.assertNotEqual(self.run_deploy('--activate', FAIL_HEALTH='1').returncode, 0)
        self.assertEqual((self.root / 'current').resolve(), self.state)
        self.assert_preserved()
        self.assertEqual((self.base / 'commands').read_text().count('sudo systemctl restart'), 2)

    def test_missing_config_and_old_node_fail_before_build(self):
        self.assertNotEqual(self.run_deploy(FAIL_NODE='1').returncode, 0)
        (self.state / 'backend/.env').unlink()
        self.assertNotEqual(self.run_deploy().returncode, 0)
        self.assertFalse(list(self.root.glob('release-*')))

if __name__ == '__main__':
    unittest.main()
