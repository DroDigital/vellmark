#!/bin/bash
# Builds Vellmark from this folder and runs it in the valley. Double-click it in Finder
# (the first time: right-click, Open). Anything after it on a command line is passed to
# the game instead, e.g. ./run-mac.command --scene yard
cd "$(dirname "$0")" || exit 1
exec ./scripts/run-from-source.sh "$@"
