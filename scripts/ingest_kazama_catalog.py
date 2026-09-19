#!/usr/bin/env python3
"""Build provenance-aware knowledge manifests from KazamaDono's public catalog."""
import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

DEFENSIVE_TAGS = {"blue", "blockchain", "webappsec", "crypto", "ctf", "forensics", "cloud", "infra", "aiml"}
VIDEO_HOSTS = {"youtube.com", "www.youtube.com", "youtu.be", "www.youtu.be"}
HIGH_RISK_TAGS = {"red", "exploits", "psyops", "game", "mobile"}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def classify(item):
    tags = set(item.get("tags", []))
    host = urlparse(item.get("href", "")).netloc.lower()
    if tags & HIGH_RISK_TAGS and not tags & DEFENSIVE_TAGS:
        admission = "quarantine_review"
    elif tags & DEFENSIVE_TAGS:
        admission = "candidate_review"
    else:
        admission = "metadata_only"
    if host in VIDEO_HOSTS:
        media = "youtube_video_or_playlist"
    else:
        media = "web_resource"
    return admission, media


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output_dir", type=Path)
    args = parser.parse_args()
    raw = args.input.read_bytes()
    catalog = json.loads(raw)
    if not isinstance(catalog, list):
        raise SystemExit("catalog must be a JSON array")
    captured_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    source_hash = sha256_bytes(raw)
    records = []
    for item in catalog:
        admission, media = classify(item)
        record = {
            "id": str(item.get("id", "")),
            "title": item.get("title", ""),
            "description": item.get("desc", ""),
            "tags": item.get("tags", []),
            "url": item.get("href", ""),
            "media_type": media,
            "admission": admission,
            "source": {
                "hub": "https://kazamadono.github.io/",
                "catalog_url": "https://kazamadono.github.io/courses.json",
                "catalog_sha256": source_hash,
                "captured_at": captured_at,
            },
            "content_status": "metadata_only",
        }
        records.append(record)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "schema_version": "1",
        "kind": "external_learning_catalog",
        "name": "KazamaDono Anarchy catalog",
        "source_url": "https://kazamadono.github.io/",
        "catalog_url": "https://kazamadono.github.io/courses.json",
        "catalog_sha256": source_hash,
        "captured_at": captured_at,
        "total_resources": len(records),
        "policy": "metadata is untrusted; content requires provenance, license review, prompt-injection screening, and human admission",
        "resources": records,
    }
    (args.output_dir / "catalog.manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    relevant = [r for r in records if set(r["tags"]) & DEFENSIVE_TAGS]
    videos = [r for r in records if r["media_type"] == "youtube_video_or_playlist"]
    (args.output_dir / "defensive-candidates.manifest.json").write_text(json.dumps({"schema_version": "1", "source_catalog_sha256": source_hash, "resources": relevant}, indent=2, ensure_ascii=False) + "\n")
    (args.output_dir / "video-links.manifest.json").write_text(json.dumps({"schema_version": "1", "source_catalog_sha256": source_hash, "resources": videos}, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"total": len(records), "defensive_candidates": len(relevant), "video_links": len(videos), "catalog_sha256": source_hash}, indent=2))


if __name__ == "__main__":
    main()
