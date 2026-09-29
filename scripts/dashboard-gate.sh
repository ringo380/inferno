#!/bin/bash
# The dashboard's gate: a clean install from the lockfile, lint, then the
# production build (which also type-checks). CI runs this on every change
# under dashboard/, so a dependency bump that breaks `npm ci` or the build
# fails its PR instead of passing green. Run it locally the same way.

set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/dashboard"

echo "==> npm ci"
npm ci --no-audit --no-fund

echo "==> npm run lint"
npm run lint

echo "==> npm run build"
npm run build
