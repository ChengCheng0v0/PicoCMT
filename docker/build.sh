#/bin/bash

TAG=${1:-latest}
docker build -t picocmt:${TAG} -f ./Dockerfile ..
