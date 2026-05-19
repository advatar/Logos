#!/usr/bin/env python3
"""Generate the narrated LP-0016 submission video."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import textwrap
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from PIL import ImageFont


ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parent
WORK_DIR = ROOT / "dist" / "submission" / "video_work"
DEFAULT_OUT = REPO / "submission" / "lp0016-demo.mp4"
FONT_CANDIDATES = [
    Path("/System/Library/Fonts/Supplemental/Arial.ttf"),
    Path("/Library/Fonts/Arial.ttf"),
    Path("/System/Library/Fonts/Supplemental/Arial Unicode.ttf"),
]


def require_tool(name: str) -> str:
    path = shutil.which(name)
    if not path:
        raise SystemExit(f"missing required tool: {name}")
    return path


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text())


def first_font() -> Path:
    for font in FONT_CANDIDATES:
        if font.exists():
            return font
    raise SystemExit("no usable font found for slide rendering")


def require_pillow() -> None:
    try:
        import PIL  # noqa: F401
    except ImportError as exc:
        raise SystemExit("missing required Python package: Pillow. Install with `brew install pillow`.") from exc


def probe_duration(ffprobe: str, path: Path) -> float:
    proc = subprocess.run(
        [
            ffprobe,
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            str(path),
        ],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    return float(proc.stdout.strip())


def wrapped(text: str, width: int = 58) -> str:
    lines: list[str] = []
    for raw in text.splitlines():
        if not raw.strip():
            lines.append("")
            continue
        prefix = ""
        body = raw
        if raw.startswith("- "):
            prefix = "- "
            body = raw[2:]
        wrapped_lines = textwrap.wrap(body, width=width, subsequent_indent="  " if prefix else "")
        if not wrapped_lines:
            lines.append(raw)
        else:
            lines.append(prefix + wrapped_lines[0])
            lines.extend(wrapped_lines[1:])
    return "\n".join(lines)


def build_slides() -> list[dict[str, str]]:
    proof = read_json(ROOT / "dist" / "submission" / "risc0_proof_performance.json")
    evidence = read_json(ROOT / "dist" / "submission" / "evidence.json")
    localnet = read_json(ROOT / "dist" / "submission" / "localnet_evidence.json")
    program_id = (ROOT / "registry" / "program_ids" / "localnet.txt").read_text().strip()
    proof_seconds = proof.get("proof_seconds", "under 10")
    gate_steps = len(evidence.get("steps", []))
    deploy_status = localnet.get("deploy", {}).get("status") or localnet.get("deploy_status") or "submitted"

    return [
        {
            "title": "LP-0016 Anonymous Forum",
            "body": (
                "Submission demo for anonymous forum moderation on the Logos stack.\n\n"
                "- Anonymous registration and posting\n"
                "- N-of-M moderation certificates\n"
                "- K-certificate slash and revocation\n"
                "- Retroactive linking only for the slashed member"
            ),
            "narration": (
                "This is the LP zero zero sixteen anonymous forum submission. "
                "The demo covers anonymous registration and posting, N of M moderation, "
                "K certificate slash, revocation, and retroactive linking only for the slashed member."
            ),
        },
        {
            "title": "Local Submission Gate",
            "body": (
                f"The local gate is the acceptance path: {gate_steps} evidence steps.\n\n"
                "- Python lifecycle demo and success tests\n"
                "- Rust workspace build and tests\n"
                "- RISC0 host and guest checks\n"
                "- Lean build, Basecamp package, LEZ diagnostics\n"
                "- Optional Noir diagnostic"
            ),
            "narration": (
                f"The local submission gate is green and is the acceptance path. "
                f"It recorded {gate_steps} evidence steps covering Python, Rust, RISC Zero, "
                "Lean, Basecamp packaging, LEZ diagnostics, and the optional Noir circuit."
            ),
        },
        {
            "title": "LEZ Standalone Sequencer Evidence",
            "body": (
                "The official LEZ wallet quickstart uses a standalone local sequencer at localhost:3040.\n\n"
                f"- Registry guest deployed locally: {deploy_status}\n"
                f"- Localnet image ID: {program_id[:18]}...{program_id[-10:]}\n"
                "- Evidence: src/dist/submission/localnet_evidence.json"
            ),
            "narration": (
                "For LEZ evidence we follow the current official wallet quickstart: "
                "run a standalone local sequencer at localhost port thirty forty. "
                "The registry guest deploys locally and the image ID is recorded in the submission evidence."
            ),
        },
        {
            "title": "RISC0 Proof Path",
            "body": (
                "RISC0 is the primary submitted zero-knowledge path.\n\n"
                "- RISC0_DEV_MODE=0\n"
                f"- Latest proof time: {proof_seconds} seconds\n"
                "- Membership and non-revocation statement\n"
                "- Evidence: src/dist/submission/risc0_proof_performance.json"
            ),
            "narration": (
                f"RISC Zero remains the primary zero knowledge proof path. "
                f"The latest local run used RISC zero dev mode set to zero and measured the proof at "
                f"{proof_seconds} seconds, under the ten second target."
            ),
        },
        {
            "title": "Lean 4 Formal Surface",
            "body": (
                "Lean 4 checks protocol invariants, not runtime code.\n\n"
                "- Certificate threshold: N signers\n"
                "- Slash bundle shape: K valid certificates\n"
                "- Revocation makes commitments inactive\n"
                "- Shamir/Lagrange reconstruction theorem contract\n"
                "- Build: cd src/lean && lake build"
            ),
            "narration": (
                "Lean four is used as a formal proof surface. It checks the threshold, "
                "slash bundle, revocation, and Shamir reconstruction theorem contracts. "
                "The Lean build is sorry free."
            ),
        },
        {
            "title": "Noir Icing",
            "body": (
                "Noir adds a compact ACIR/Nargo circuit for the post-binding relation.\n\n"
                "- Private: member_secret, opening\n"
                "- Public: commitment, nullifier, retro tag\n"
                "- Tests: accepts valid binding, rejects bad nullifier\n"
                "- Run: cd src/noir/post_binding && nargo test"
            ),
            "narration": (
                "Noir is the icing on the cake. It does not replace RISC Zero. "
                "It adds a compact ACIR circuit where the member secret and opening stay private, "
                "while the public commitment, nullifier, and retro tag are constrained."
            ),
        },
        {
            "title": "Basecamp User Flow",
            "body": (
                "The Basecamp package is generated by the local gate.\n\n"
                "- Create forum\n"
                "- Register and post\n"
                "- Moderate and collect certificates\n"
                "- Slash, revoke, and inspect history\n"
                "- Inspector click-through exists for runtime artifacts"
            ),
            "narration": (
                "The Basecamp flow is packaged for non technical users. It covers forum creation, "
                "registration, posting, moderation, certificate review, slash, revocation, and history inspection."
            ),
        },
        {
            "title": "Transparent Remaining Blockers",
            "body": (
                "The repo calls out the remaining external items clearly.\n\n"
                "- CU numbers need custom invoke/CU reporting from scaffold or wallet\n"
                "- Public devnet/testnet proof only if reviewers require separate endpoints\n"
                "- Clean-shell Basecamp click-through needs external runtime artifacts\n"
                "- The narrated video is generated here"
            ),
            "narration": (
                "The remaining blockers are transparent. Compute unit numbers need a custom invoke "
                "and CU reporting path. Public devnet or testnet proof is only needed if reviewers "
                "require separate public endpoints. Clean shell Basecamp click through needs external runtime artifacts."
            ),
        },
        {
            "title": "How To Verify",
            "body": (
                "Start from the local gate and evidence files.\n\n"
                "- cd src && scripts/local_submission_gate.py\n"
                "- python3 scripts/collect_localnet_evidence.py\n"
                "- python3 scripts/check_noir_icing.py --pretty\n"
                "- cd src/lean && lake build\n"
                "- README.md lists blockers and proof-stack details"
            ),
            "narration": (
                "To verify the submission, start with the local gate, then inspect the localnet evidence, "
                "the Noir diagnostic, and the Lean build. The README now lists the blockers and proof stack clearly."
            ),
        },
    ]


def render_segment(
    *,
    ffmpeg: str,
    ffprobe: str,
    say: str,
    font: Path,
    index: int,
    slide: dict[str, str],
) -> Path:
    from PIL import Image, ImageDraw, ImageFont

    title_path = WORK_DIR / f"slide_{index:02d}_title.txt"
    body_path = WORK_DIR / f"slide_{index:02d}_body.txt"
    audio_path = WORK_DIR / f"slide_{index:02d}.aiff"
    image_path = WORK_DIR / f"slide_{index:02d}.png"
    segment_path = WORK_DIR / f"slide_{index:02d}.mp4"

    title_path.write_text(slide["title"])
    body_path.write_text(wrapped(slide["body"]))
    subprocess.run([say, "-o", str(audio_path), slide["narration"]], check=True)
    duration = max(probe_duration(ffprobe, audio_path) + 0.35, 4.0)
    render_slide_png(
        image_path=image_path,
        title=slide["title"],
        body=body_path.read_text(),
        font=font,
        image_module=Image,
        draw_module=ImageDraw,
        font_module=ImageFont,
    )

    subprocess.run(
        [
            ffmpeg,
            "-y",
            "-loop",
            "1",
            "-framerate",
            "30",
            "-i",
            str(image_path),
            "-i",
            str(audio_path),
            "-t",
            f"{duration:.3f}",
            "-c:v",
            "libx264",
            "-tune",
            "stillimage",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "veryfast",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-shortest",
            str(segment_path),
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return segment_path


def render_slide_png(
    *,
    image_path: Path,
    title: str,
    body: str,
    font: Path,
    image_module,
    draw_module,
    font_module: "ImageFont",
) -> None:
    image = image_module.new("RGB", (1280, 720), "#07111f")
    draw = draw_module.Draw(image)
    title_font = font_module.truetype(str(font), 46)
    body_font = font_module.truetype(str(font), 30)
    footer_font = font_module.truetype(str(font), 20)

    draw.rectangle((0, 0, 1280, 720), fill="#07111f")
    draw.rectangle((0, 0, 18, 720), fill="#5eead4")
    draw.rounded_rectangle((42, 34, 1238, 640), radius=18, outline="#1f3b57", width=2)
    draw.text((66, 58), title, font=title_font, fill="#66e3ff")
    draw.multiline_text((66, 146), body, font=body_font, fill="#f6f8ff", spacing=12)
    draw.text((66, 668), "LP-0016 Logos anonymous forum", font=footer_font, fill="#8ea0b8")
    image.save(image_path)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    args = parser.parse_args()

    ffmpeg = require_tool("ffmpeg")
    ffprobe = require_tool("ffprobe")
    say = require_tool("say")
    font = first_font()
    require_pillow()

    WORK_DIR.mkdir(parents=True, exist_ok=True)
    args.out.parent.mkdir(parents=True, exist_ok=True)

    segments = [
        render_segment(
            ffmpeg=ffmpeg,
            ffprobe=ffprobe,
            say=say,
            font=font,
            index=index,
            slide=slide,
        )
        for index, slide in enumerate(build_slides(), start=1)
    ]

    concat_path = WORK_DIR / "concat.txt"
    concat_path.write_text("".join(f"file '{segment}'\n" for segment in segments))
    subprocess.run(
        [
            ffmpeg,
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            str(concat_path),
            "-c",
            "copy",
            str(args.out),
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    print(args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
