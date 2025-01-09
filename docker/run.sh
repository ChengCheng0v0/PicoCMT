#/bin/bash

NAME=${NAME:-picocmt}
TAG=${TAG:-latest}
ONLY_SERVER=${ONLY_SERVER:-0}
PUBLISH_SERVER_PORT=${PUBLISH_SERVER_PORT:-3000}
PUBLISH_CLIENT_RES_PORT=${PUBLISH_CLIENT_RES_PORT:-80}
CONFIG_PATH=${CONFIG_PATH:-/etc/picocmt/config.toml}

docker run --name ${NAME} --mount type=bind,source=${CONFIG_PATH},target=/app/server/config.toml -e ONLY_SERVER=${ONLY_SERVER} --publish ${PUBLISH_SERVER_PORT}:3000 --publish ${PUBLISH_CLIENT_RES_PORT}:80 picocmt:${TAG}

# Example
# PUBLISH_SERVER_PORT=3001 PUBLISH_CLIENT_RES_PORT=81 ./run.sh
