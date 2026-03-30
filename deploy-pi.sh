#!/bin/bash
set -e

PI_HOST="colinr@home-ihd.local"
PI_APP_DIR="~/agile-fetcher"

echo "==> Syncing project to Pi..."
rsync -avz \
  --exclude target \
  --exclude .git \
  --exclude node_modules \
  ./ "${PI_HOST}:${PI_APP_DIR}"

echo "==> Building on Pi..."
ssh "${PI_HOST}" "
  source \$HOME/.cargo/env &&
  cd ${PI_APP_DIR} &&
  cargo build --release
"

echo "==> Restarting ihd service..."
ssh "${PI_HOST}" "
  sudo systemctl restart ihd &&
  sudo systemctl status ihd --no-pager
"

echo "==> Done."