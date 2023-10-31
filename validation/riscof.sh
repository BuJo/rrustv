#!/bin/bash

DIR=`dirname $(readlink -f $0)`
if which docker >/dev/null 2>&1; then
  RUNNER=docker
elif which podman >/dev/null 2>&1; then
  RUNNER=podman
else
  exit 1
fi

$RUNNER run -it --rm -v $DIR:/work:z -v $DIR/../target/:/target/:z jbuch/riscvvalidation "$@"
