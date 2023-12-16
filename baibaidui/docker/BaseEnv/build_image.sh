#!/bin/bash

PRJ_NAME=`python3 scripts/saaf/project_name.py`
SUBFIX=_env

cat > docker/BaseEnv/Dockerfile <<'END'

FROM ubuntu:18.04

LABEL maintainers="ActivePeter"

RUN apt-get update && apt-get install -y python3 python3-pip git iproute2 iputils-ping && mkdir -p /tmp/install

ENTRYPOINT ["echo","helloworld"]

END

docker build -t $PRJ_NAME$SUBFIX:v1 docker/BaseEnv --no-cache

rm -f docker/BaseEnv/Dockerfile