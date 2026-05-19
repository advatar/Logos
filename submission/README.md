# Submission Video

`lp0016-demo.mp4` is the narrated LP-0016 demo video generated from local
evidence. It covers the protocol lifecycle, local submission gate, LEZ
standalone sequencer evidence, RISC0 proof path, Lean 4 proof surface, optional
Noir circuit, Basecamp flow, and transparent remaining external blockers.

Regenerate it with:

```bash
cd src
python3 scripts/make_submission_video.py
```

Local generation requires `ffmpeg`, `ffprobe`, macOS `say`, and Pillow.
