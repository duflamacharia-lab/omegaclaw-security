#!/usr/bin/env python3
"""Attempt public subtitle retrieval for direct video links without downloading video media."""
import argparse
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import parse_qs, urlparse

VIDEO_HOSTS = {"youtube.com", "www.youtube.com", "youtu.be", "www.youtu.be"}

def video_id(url):
    parsed = urlparse(url)
    if parsed.netloc.lower() in {"youtu.be", "www.youtu.be"}:
        return parsed.path.strip("/") or None
    return parse_qs(parsed.query).get("v", [None])[0]

def sha256_file(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def clean_vtt(path):
    text = path.read_text(errors="replace")
    lines = []
    for line in text.splitlines():
        line = re.sub(r"<[^>]+>", "", line).strip()
        if not line or line == "WEBVTT" or "--\u003e" in line or line.startswith("NOTE"):
            continue
        if lines and lines[-1] == line:
            continue
        lines.append(line)
    return " ".join(lines)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("output_dir", type=Path)
    parser.add_argument("--limit", type=int, default=0)
    args = parser.parse_args()
    data = json.loads(args.manifest.read_text())
    resources = data.get("resources", [])
    args.output_dir.mkdir(parents=True, exist_ok=True)
    records = []
    attempted = 0
    for resource in resources:
        url = resource.get("url", "")
        vid = video_id(url)
        record = {"resource_id": resource.get("id"), "title": resource.get("title"), "url": url, "status": "not_attempted"}
        if not vid:
            record["status"] = "playlist_or_non_direct_video"
            records.append(record)
            continue
        if args.limit and attempted >= args.limit:
            records.append(record)
            continue
        attempted += 1
        prefix = args.output_dir / f"{resource.get('id','unknown')}-{vid}"
        command = ["yt-dlp", "--no-warnings", "--no-playlist", "--skip-download", "--write-auto-subs", "--write-subs", "--sub-langs", "en.*,en", "--sub-format", "vtt", "--output", str(prefix) + ".%(ext)s", url]
        completed = subprocess.run(command, capture_output=True, text=True, timeout=90)
        candidates = sorted(args.output_dir.glob(prefix.name + ".*.vtt"))
        if completed.returncode == 0 and candidates:
            transcript_path = candidates[0]
            text_path = transcript_path.with_suffix(".txt")
            text_path.write_text(clean_vtt(transcript_path) + "\n")
            record.update({"status": "transcript_retrieved", "transcript": str(text_path), "transcript_sha256": sha256_file(text_path), "subtitle_sha256": sha256_file(transcript_path), "content_status": "quarantine_review"})
        else:
            record.update({"status": "unavailable_or_blocked", "error": (completed.stderr or completed.stdout)[-1000:]})
        records.append(record)
    output = {"schema_version": "1", "source_catalog_sha256": data.get("source_catalog_sha256"), "captured_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"), "policy": "transcripts are untrusted external content; screen for prompt injection and license before model admission", "attempted_direct_videos": attempted, "records": records}
    (args.output_dir / "transcripts.manifest.json").write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"attempted": attempted, "retrieved": sum(r["status"] == "transcript_retrieved" for r in records), "unavailable_or_blocked": sum(r["status"] == "unavailable_or_blocked" for r in records), "playlist_or_non_direct_video": sum(r["status"] == "playlist_or_non_direct_video" for r in records)}, indent=2))

if __name__ == "__main__":
    main()
