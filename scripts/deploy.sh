#!/usr/bin/env bash
# Build an isolated release; optionally switch preconfigured systemd services.
set -Eeuo pipefail
umask 027
fail() { echo "Error: $*" >&2; exit 1; }
[[ $# -ge 2 && $# -le 3 ]] || fail "Usage: $0 STATE_CHECKOUT RELEASE_ROOT [--activate]"
mode=${3:---build-only}
[[ $mode == --build-only || $mode == --activate ]] || fail "Unknown mode: $mode"
source_dir=$(cd "$(dirname "$0")/.." && pwd -P)
state=$(realpath -e "$1")
mkdir -p "$2"
root=$(realpath -e "$2")
[[ $root != "$state" && $root != "$state/"* && $state != "$root/"* ]] || fail "State and releases must be separate directories"
for cmd in git node npm cargo flock curl; do command -v "$cmd" >/dev/null || fail "Missing $cmd"; done
node -e 'const [a,b,c]=process.versions.node.split(".").map(Number); if(a!==24 || b<22 || (b===22 && c<1)) process.exit(1)' || fail "Use Node 24.22.1 or newer 24.x"
git -C "$source_dir" diff --quiet HEAD -- || fail "Commit tracked source changes before building"
exec 9>"$root/.deploy.lock"
flock -n 9 || fail "Another deployment is running"
for file in backend/.env backend/.env.release frontend/.env frontend/.env.production backend/src/gazetteer_load/2021_Gaz_zcta_national.txt; do
  [[ -s $state/$file ]] || fail "Missing prerequisite: $state/$file (see server_admin/README.md)"
done
head -n 1 "$state/backend/src/gazetteer_load/2021_Gaz_zcta_national.txt" | grep -q GEOID || fail "Gazetteer header invalid (test fixtures are not deployable)"
[[ -d $state/frontend/public/data ]] || fail "Missing public/data; provision or explicitly create an empty media directory"
previous=''
if [[ $mode == --activate ]]; then
  command -v sqlite3 >/dev/null || fail "Missing sqlite3"
  [[ -L $root/current && -d $root/current ]] || fail "Configure current symlink and services first"
  previous=$(readlink -f "$root/current")
  [[ $(systemctl show -p ExecStart --value frontend_fruitfacts) == *"$(command -v node) $root/current/frontend/node_modules/next/dist/bin/next"* ]] || fail "Frontend ExecStart must use the tested Node executable and release Next.js binary"
  [[ $(systemctl show -p WorkingDirectory --value frontend_fruitfacts) == "$root/current/frontend" ]] || fail "Frontend service WorkingDirectory must use $root/current/frontend"
  [[ $(systemctl show -p WorkingDirectory --value backend_fruitfacts) == "$state/backend" ]] || fail "Backend service WorkingDirectory must remain $state/backend"
  [[ $(systemctl show -p ExecStart --value backend_fruitfacts) == *"$root/current/backend/harvest-chart-server"* ]] || fail "Backend ExecStart must use the release symlink"
  [[ -s $state/backend/database.sqlite3 ]] || fail "Missing database: restore/provision separately; deployment never imports or resets it"
  [[ $(sqlite3 -readonly "$state/backend/database.sqlite3" 'PRAGMA quick_check;') == ok ]] || fail "SQLite integrity check failed"
fi
revision=$(git -C "$source_dir" rev-parse HEAD)
release=$(mktemp -d "$root/release-${revision:0:12}-XXXXXX")
# Allow the existing deployment/service group to traverse this newly created release.
chmod 750 "$release"
switched=false
rollback() {
  status=$?
  trap - EXIT
  if $switched && (( status != 0 )); then
    echo "Activation failed; restoring $previous. Failed release retained: $release" >&2
    ln -s "$previous" "$root/.rollback-$$"
    mv -Tf "$root/.rollback-$$" "$root/current"
    sudo systemctl restart backend_fruitfacts frontend_fruitfacts || echo "Rollback restart failed; inspect systemd immediately" >&2
  fi
  exit "$status"
}
trap rollback EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
# No remote fetch, reset, data reload, ownership changes, or certificate operations.
git clone --quiet --no-hardlinks --local "$source_dir" "$release"
git -C "$release" checkout --quiet --detach "$revision"
for file in backend/.env backend/.env.debug backend/.env.release frontend/.env frontend/.env.production frontend/.env.local frontend/.env.production.local; do
  [[ ! -f $state/$file ]] || cp -p "$state/$file" "$release/$file"
done
cp "$state/backend/src/gazetteer_load/2021_Gaz_zcta_national.txt" "$release/backend/src/gazetteer_load/"
# Only ignored media is shared; generated code and assets stay in this release.
mv "$release/frontend/public/data" "$release/media-manifests"
ln -s "$state/frontend/public/data" "$release/frontend/public/data"
export NEXT_TELEMETRY_DISABLED=1
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
(cd "$release/frontend" && npm ci --include=dev && npm test && npm run build)
(cd "$release/backend" && cargo test --locked --no-default-features --lib && cargo build --locked --release --no-default-features --bin harvest-chart-server)
cp "$release/backend/target/release/harvest-chart-server" "$release/backend/harvest-chart-server"
echo "Built $revision at $release"
if [[ $mode == --activate ]]; then
  # SQLite's online backup includes WAL data; never copy an open database file.
  (cd "$release" && sqlite3 "$state/backend/database.sqlite3" '.backup database-before-activation.sqlite3')
  ln -s "$release" "$root/.current-$$"
  switched=true
  mv -Tf "$root/.current-$$" "$root/current"
  sudo systemctl restart backend_fruitfacts frontend_fruitfacts
  healthy=false
  for ((i=0; i<30; i++)); do
    if systemctl is-active --quiet backend_fruitfacts &&
       systemctl is-active --quiet frontend_fruitfacts &&
       curl --fail --silent --max-time 2 http://127.0.0.1:3001/api/fact >/dev/null &&
       curl --fail --silent --max-time 2 http://127.0.0.1:3000/plants >/dev/null; then
      healthy=true; break
    fi
    sleep 1
  done
  $healthy || fail "Local health checks failed"
  printf '%s\n' "$previous" > "$release/previous-release"
  switched=false
  echo "Activated $release; previous release: $previous"
fi
