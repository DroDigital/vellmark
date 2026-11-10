#!/usr/bin/env python3
"""Generate Vellmark voice lines with a hard credit limit.

Nothing is sent to ElevenLabs unless --generate is given.

  python3 tools/generate_voice.py                         show the budget
  python3 tools/generate_voice.py --generate --priority P1
  python3 tools/generate_voice.py --generate --only Q3_001
  python3 tools/generate_voice.py --generate --retake Q3_001

Needs the ELEVENLABS_API_KEY environment variable and docs/voice/voice_ids.json.
Finished lines are skipped, so an interrupted run can simply be started again.
"""
import argparse
import csv
import json
import os
import pathlib
import sys
import urllib.error
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
LINES = ROOT / "docs" / "voice" / "voice_lines.tsv"
VOICE_IDS = ROOT / "docs" / "voice" / "voice_ids.json"
LEDGER = ROOT / "docs" / "voice" / "credit_ledger.tsv"
OUT = ROOT / "third_party" / "inbox" / "voice"
MODEL = "eleven_v4"
FORMAT = "mp3_44100_128"
LIMIT = 36000  # The account has 40,000. The rest covers voice creation and calibration.


def load_lines():
    with LINES.open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t", quoting=csv.QUOTE_NONE))


def spent():
    if not LEDGER.exists():
        return 0
    with LEDGER.open(encoding="utf-8", newline="") as handle:
        return sum(int(row["credits"]) for row in csv.DictReader(handle, delimiter="\t"))


def record(line_id, take, credits, path):
    new = not LEDGER.exists()
    with LEDGER.open("a", encoding="utf-8", newline="") as handle:
        if new:
            handle.write("id\ttake\tcredits\tfile\n")
        handle.write(f"{line_id}\t{take}\t{credits}\t{path.relative_to(ROOT).as_posix()}\n")


def takes(line_id):
    return sorted(OUT.glob(f"{line_id}.take*.mp3"))


def synthesize(voice_id, text, key):
    base = os.environ.get("ELEVENLABS_BASE_URL", "https://api.elevenlabs.io")
    request = urllib.request.Request(
        f"{base}/v1/text-to-speech/{voice_id}?output_format={FORMAT}",
        data=json.dumps({"text": text, "model_id": MODEL}).encode("utf-8"),
        headers={"xi-api-key": key, "Content-Type": "application/json", "Accept": "audio/mpeg"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=120) as response:
        audio = response.read()
        header = response.headers.get("character-cost")
    return audio, int(header) if header else len(text)


def report(lines):
    print(f"Credits recorded as spent: {spent()} of {LIMIT} allowed")
    for priority in sorted({row["priority"] for row in lines}):
        group = [row for row in lines if row["priority"] == priority]
        todo = [row for row in group if not takes(row["id"])]
        print(f"{priority}: {len(group)} lines, {sum(len(r['text']) for r in group)} characters, "
              f"{sum(len(r['text']) for r in todo)} characters not yet generated")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--generate", action="store_true", help="spend credits")
    parser.add_argument("--priority", help="P1, P2, or P3")
    parser.add_argument("--only", help="one line ID")
    parser.add_argument("--retake", help="generate another take of one line ID")
    parser.add_argument("--limit", type=int, default=LIMIT)
    args = parser.parse_args()

    lines = load_lines()
    if not args.generate:
        report(lines)
        return

    key = os.environ.get("ELEVENLABS_API_KEY")
    if not key:
        sys.exit("Set ELEVENLABS_API_KEY first.")
    if not VOICE_IDS.exists():
        sys.exit("Create docs/voice/voice_ids.json first. See voice_ids.example.json.")
    voices = json.loads(VOICE_IDS.read_text(encoding="utf-8"))
    OUT.mkdir(parents=True, exist_ok=True)

    if args.retake:
        chosen = [row for row in lines if row["id"] == args.retake]
    elif args.only:
        chosen = [row for row in lines if row["id"] == args.only]
    else:
        chosen = [row for row in lines if not args.priority or row["priority"] == args.priority]
    if not chosen:
        sys.exit("No matching lines.")

    total = spent()
    for row in chosen:
        existing = takes(row["id"])
        if existing and not args.retake:
            continue
        if total + len(row["text"]) > args.limit:
            print(f"Stopping before {row['id']}: the limit of {args.limit} credits would be passed.")
            break
        try:
            audio, credits = synthesize(voices[row["base"]], row["text"], key)
        except KeyError:
            sys.exit(f"No voice ID for {row['base']}.")
        except urllib.error.HTTPError as error:
            sys.exit(f"ElevenLabs returned {error.code} for {row['id']}. Nothing more was sent.")
        take = len(existing) + 1
        path = OUT / f"{row['id']}.take{take}.mp3"
        path.write_bytes(audio)
        total += credits
        record(row["id"], take, credits, path)
        print(f"{row['id']} take {take}: {credits} credits, {total} spent")


if __name__ == "__main__":
    main()
