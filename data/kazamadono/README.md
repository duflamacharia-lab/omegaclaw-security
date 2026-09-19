# KazamaDono catalog snapshot

This directory contains a provenance-tracked snapshot of the public [KazamaDono Anarchy catalog](https://kazamadono.github.io/).

- `courses.json` is the captured source catalog.
- `catalog.manifest.json` indexes all 1,708 resources.
- `defensive-candidates.manifest.json` contains 998 candidate resources selected by defensive and engineering tags.
- `video-links.manifest.json` contains 101 YouTube-linked records.
- `transcripts/transcripts.manifest.json` records the transcript attempt and its outcomes.
- `transcripts/*.txt` contains 9 cleaned transcript artifacts retrieved from direct video links.

All content is untrusted. Transcript artifacts are marked `quarantine_review` and must pass licensing, prompt-injection, executable-content, and defensive-relevance review before use in training, retrieval, or trusted memory.

Rebuild the metadata manifests with:

```bash
python3 scripts/ingest_kazama_catalog.py data/kazamadono/courses.json data/kazamadono
```

Retry public subtitle retrieval with:

```bash
python3 scripts/collect_kazama_transcripts.py data/kazamadono/video-links.manifest.json data/kazamadono/transcripts
```

The collector records unavailable and blocked resources. It does not download video media and does not treat a playlist URL as a transcript.
