#!/usr/bin/env bash
# Run a command inside the SDK image `prismpm.lock` pins, exactly as
# .github/workflows/build.yml does: the checkout bind-mounted at its own path,
# a private /tmp, and the host's Docker socket for prismpm's nested builds.
#   scripts/sdk.sh 'prismpm --project app verify'
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
image=$(sed -n 's/.*"sdk_image":"\([^"]*\)".*/\1/p' prismpm.lock)
[[ $image =~ @sha256:[0-9a-f]{64}$ ]] || { echo 'prismpm.lock does not select an immutable SDK image' >&2; exit 64; }
scratch=${PRISM_SDK_SCRATCH:-$(mktemp -d)}
mkdir -p "$scratch/prismpm-home/.docker"
[[ -f $scratch/prismpm-home/.docker/config.json ]] || printf '{}\n' >"$scratch/prismpm-home/.docker/config.json"
# Behind an egress proxy with its own CA (a sandboxed agent session), pass both
# through; a GitHub runner has neither, so CI runs exactly as build.yml does.
extra=()
if [[ -n ${HTTPS_PROXY:-} && -n ${SSL_CERT_FILE:-} && -f ${SSL_CERT_FILE:-/nonexistent} ]]; then
  extra=(--network host --volume "$SSL_CERT_FILE:/etc/prism-proxy-ca.crt:ro"
    --env HTTPS_PROXY="$HTTPS_PROXY" --env https_proxy="$HTTPS_PROXY"
    --env CARGO_HTTP_CAINFO=/etc/prism-proxy-ca.crt --env SSL_CERT_FILE=/etc/prism-proxy-ca.crt
    --env NODE_EXTRA_CA_CERTS=/etc/prism-proxy-ca.crt --env GIT_SSL_CAINFO=/etc/prism-proxy-ca.crt)
fi
exec docker run --rm "${extra[@]}" \
  --user "$(id -u):$(id -g)" \
  --group-add "$(stat -c '%g' /var/run/docker.sock)" \
  --volume "$scratch:/tmp" \
  --volume "$root:$root" \
  --volume /var/run/docker.sock:/var/run/docker.sock \
  --workdir "$root" \
  --env DOCKER_CONFIG=/tmp/prismpm-home/.docker \
  --env BUILDX_CONFIG=/tmp/prismpm-buildx \
  --env HOME=/tmp/prismpm-home \
  --env PRISMPM_EPHEMERAL_HOME=1 \
  "$image" sh -c "$*"
