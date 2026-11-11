#!/bin/bash
# The performance run. Double-click this file. It flies the tour's camera path twice
# and writes vellmark-perf-report.txt to your Desktop. Send that file back.
cd "$(dirname "$0")" || exit 1
# A build downloaded from the web is quarantined by macOS; this lets it start.
xattr -dr com.apple.quarantine Vellmark.app 2>/dev/null
REPORT="$HOME/Desktop/vellmark-perf-report.txt"
echo "Running the Vellmark performance run. A window will open for about a minute."
echo "Leave the Mac alone until it closes."
./Vellmark.app/Contents/MacOS/vellmark --perf-run --out "$REPORT"
echo
if [ -f "$REPORT" ]; then
  echo "Done. The report is on your Desktop: vellmark-perf-report.txt"
  open -R "$REPORT"
else
  echo "The run did not write a report. Please send back this window's text instead."
fi
