"""Measures crisis-moments on the clips of clips.lock.json and prints a table to paste in the README.

For each clip it makes a video of the sound (FFmpeg), runs the example flow (speech-to-text,
sound-events and sound-volume feeding crisis-moments) through `metil run`, and counts a moment found
as a "flag". A clip a person should look at (`shouldFlag`) that gets a flag is a hit; one that gets
none is a miss; a clip a person should not look at that gets a flag is a false alarm.

Usage: python evaluate.py <folder with the clips> <metil program> <languages, like en,pt,es,fr,de>
[settings of crisis-moments.find as JSON, like '{"threshold": 0.4}']
Needs FFmpeg, the plugins built (cargo build --release for the three Rust ones), and a Python with
faster-whisper first on PATH for speech-to-text.
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).parent
PLUGINS = HERE.parent.parent  # the apps folder
FLOW = HERE.parent / "examples" / "moments-to-review.mtf"


def make_video(source: Path, target: Path) -> None:
    """A video that has the sound of `source` (at most 60 seconds) and a black picture."""
    subprocess.run(
        [
            "ffmpeg", "-nostdin", "-hide_banner", "-loglevel", "error", "-y",
            "-f", "lavfi", "-i", "color=c=black:s=320x180:r=5:d=60", "-i", str(source),
            "-t", "60", "-shortest", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac",
            str(target),
        ],
        check=True,
    )


def run_flow(
    metil: str, video: Path, languages: list[str], settings: dict, work: Path
) -> list[dict]:
    """The moments the flow finds in `video`; an empty list when it finds none."""
    flow = json.loads(FLOW.read_text(encoding="utf-8"))
    flow["actions"]["Moments"]["inputs"]["languages"] = languages
    flow["actions"]["Moments"]["inputs"].update(settings)
    flow["triggers"]["t"]["inputs"]["start"] = [{"path": str(video)}]
    flow_file = work / "flow.mtf"
    flow_file.write_text(json.dumps(flow), encoding="utf-8")
    result = work / "result.json"
    command = [metil, "run", str(flow_file), "--out", str(result)]
    for plugin in ("speech-to-text", "sound-events", "sound-volume", "crisis-moments"):
        command += ["--plugins", str(PLUGINS / plugin)]
    done = subprocess.run(command, capture_output=True, text=True, timeout=900, check=False)
    if not result.exists():
        raise RuntimeError(f"the flow failed on {video.name}: {(done.stdout + done.stderr)[-400:]}")
    records = json.loads(result.read_text(encoding="utf-8"))
    return records[0].get("moments") or []


def main() -> None:
    folder, metil = Path(sys.argv[1]), sys.argv[2]
    languages = sys.argv[3].split(",") if len(sys.argv) > 3 else ["en"]
    settings = json.loads(sys.argv[4]) if len(sys.argv) > 4 else {}
    clips = json.loads((HERE / "clips.lock.json").read_text(encoding="utf-8"))
    rows = []
    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        for clip in clips:
            source = folder / clip["file"]
            if not source.exists():
                continue
            video = work / f"{clip['name']}.mp4"
            make_video(source, video)
            moments = run_flow(metil, video, languages, settings, work)
            rows.append((clip, moments))
            labels = ", ".join(f"{m['label']} {m['confidence']:.2f}" for m in moments) or "none"
            print(f"{clip['name']:16} should flag: {str(clip['shouldFlag']):5}  found: {labels}", flush=True)
    hits = sum(1 for clip, moments in rows if clip["shouldFlag"] and moments)
    misses = sum(1 for clip, moments in rows if clip["shouldFlag"] and not moments)
    false_alarms = sum(1 for clip, moments in rows if not clip["shouldFlag"] and moments)
    quiet = sum(1 for clip, moments in rows if not clip["shouldFlag"] and not moments)
    print(f"\nlanguages {languages}: {len(rows)} clips; hits {hits}, misses {misses}, "
          f"false alarms {false_alarms}, correctly quiet {quiet}")


if __name__ == "__main__":
    main()
