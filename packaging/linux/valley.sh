#!/bin/sh
# Opens Vellmark in the valley, standing on Tithe Hill, instead of the yard.
cd "$(dirname "$0")" || exit 1
exec ./vellmark --scene valley "$@"
