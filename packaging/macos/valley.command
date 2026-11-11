#!/bin/bash
# Opens Vellmark in the valley of Phase S02, standing on Tithe Hill, instead of the yard.
# Double-click this file.
cd "$(dirname "$0")" || exit 1
# A build downloaded from the web is quarantined by macOS; this lets it start.
xattr -dr com.apple.quarantine Vellmark.app 2>/dev/null
./Vellmark.app/Contents/MacOS/vellmark --scene valley
