#!/bin/sh
# Builds Vellmark from this folder and runs it in the valley. Anything given to it is
# passed to the game instead, e.g. ./run-linux.sh --scene yard
cd "$(dirname "$0")" || exit 1
exec ./scripts/run-from-source.sh "$@"
