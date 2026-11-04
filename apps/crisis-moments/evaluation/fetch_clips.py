"""Fetches the test clips of the crisis-moments evaluation and writes clips.lock.json.

Each clip is the first result of a YouTube search that the site marks as Creative Commons and that is
at most 90 seconds long. Only the audio is downloaded, into the folder given on the command line (it
is not kept in the repository); the lock file keeps where each clip came from, so anyone can fetch
the same ones. Needs yt-dlp. Usage: python fetch_clips.py <folder for the clips>
"""

import json
import re
import subprocess
import sys
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).parent

#: One clip for each case: the name, what to search for, and whether a person should look at it.
#: Sounds of danger that are not a person (an alarm, a siren, gunshots, an explosion) are not what the
#: plugin looks for, so they are clips it must leave alone.
CASES = [
    ("scream", "woman screaming sound effect", True),
    ("crying", "crying sound effect", True),
    ("sobbing", "woman sobbing sound effect", True),
    ("heartbeat", "dramatic heartbeat sound effect", False),
    ("punch", "punch hit sound effect", True),
    ("argument", "angry shouting argument", True),
    ("alarm", "fire alarm sound effect", False),
    ("siren", "ambulance siren sound", False),
    ("gunshot", "gunshot sound effect", False),
    ("explosion", "explosion sound effect", False),
    ("stadium", "football stadium crowd", False),
    ("rock", "rock band live music", False),
    ("laughter", "laughing laughter", False),
]

#: Clips from Wikimedia Commons (free licences), cut to their first 60 seconds: the name, the file
#: on Commons and whether a person should look at it. Used for what the search finds too few of.
COMMONS = [
    ("piano-1920s", "File:\"It all depends on you\" by Franklyn Baur and Jesse Crawford.wav", False),
    ("spoken-article", "File:En-Butterfly(Loona song)-article.ogg", False),
    ("brass-band", "File:March Grandioso - Concert Band - United States Air Force Band of the Rockies.mp3", False),
    ("applause", "File:277021 sandermotions applause-2.wav", False),
    ("thunder", "File:Thunder.wav", False),
]


def fetch(case: tuple[str, str, bool], folder: Path) -> dict | None:
    """Searches and downloads the clip of one case; `None` when no Creative Commons clip was found."""
    name, term, flag = case
    target = folder / f"{name}.%(ext)s"
    done = subprocess.run(
        [
            "yt-dlp", "--ignore-errors", "--no-warnings", "--max-downloads", "1",
            "--match-filter", "license*=Creative Commons & duration<=90",
            "-f", "bestaudio", "-o", str(target),
            "--print", "after_move:%(.{id,duration,license,uploader,title,filepath})j",
            f"ytsearch40:{term}",
        ],
        capture_output=True, text=True, timeout=900, check=False,
    )
    for line in done.stdout.splitlines():
        try:
            found = json.loads(line)
        except ValueError:
            continue
        if len(found.get("id", "")) == 11:
            return {
                "name": name, "search": term, "shouldFlag": flag,
                "url": f"https://www.youtube.com/watch?v={found['id']}", "title": found["title"],
                "uploader": found["uploader"], "licence": found["license"],
                "seconds": float(found["duration"]), "file": Path(found["filepath"]).name,
            }
    return None


def fetch_commons(case: tuple[str, str, bool], folder: Path) -> dict | None:
    """Downloads one file of Wikimedia Commons, with its licence; `None` when it cannot be fetched."""
    name, title, flag = case
    headers = {"User-Agent": "loomas-crisis-eval/0.1 (https://github.com/loomas-apps/LoomasPlugins)"}
    query = urllib.parse.urlencode(
        {"action": "query", "titles": title, "prop": "imageinfo", "format": "json",
         "iiprop": "url|extmetadata"}
    )
    request = urllib.request.Request(f"https://commons.wikimedia.org/w/api.php?{query}", headers=headers)
    try:
        page = next(iter(json.load(urllib.request.urlopen(request, timeout=60))["query"]["pages"].values()))
        info = page["imageinfo"][0]
        extra = info.get("extmetadata", {})
        suffix = Path(urllib.parse.urlparse(info["url"]).path).suffix
        source = folder / f"{name}{suffix}"
        download = urllib.request.Request(info["url"], headers=headers)
        source.write_bytes(urllib.request.urlopen(download, timeout=120).read())
    except (OSError, KeyError, StopIteration, ValueError) as error:
        print(f"{name}: not fetched ({error})")
        return None
    return {
        "name": name, "search": title, "shouldFlag": flag,
        "url": "https://commons.wikimedia.org/wiki/" + urllib.parse.quote(title.replace(" ", "_")),
        "title": title, "uploader": re.sub("<[^>]+>", "", extra.get("Artist", {}).get("value", ""))[:60],
        "licence": extra.get("LicenseShortName", {}).get("value", "unknown"), "seconds": 60.0,
        "file": source.name,
    }


def restore(clip: dict, folder: Path) -> None:
    """Downloads again a clip of the lock file whose file is not in `folder` (by its address)."""
    if (folder / clip["file"]).exists():
        return
    if clip["url"].startswith("https://www.youtube.com/"):
        target = folder / f"{clip['name']}.%(ext)s"
        subprocess.run(
            ["yt-dlp", "--ignore-errors", "--no-warnings", "-f", "bestaudio", "-o", str(target), clip["url"]],
            capture_output=True, text=True, timeout=600, check=False,
        )
    elif clip["url"].startswith("https://commons.wikimedia.org/"):
        fetch_commons((clip["name"], clip["title"], clip["shouldFlag"]), folder)


def main() -> None:
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    lock = HERE / "clips.lock.json"
    kept = {clip["name"]: clip for clip in json.loads(lock.read_text(encoding="utf-8"))} if lock.exists() else {}
    for clip in list(kept.values()):
        restore(clip, folder)
    with ThreadPoolExecutor(max_workers=5) as pool:
        fetched = list(pool.map(lambda case: fetch(case, folder), [c for c in CASES if c[0] not in kept]))
        free = list(pool.map(lambda case: fetch_commons(case, folder), [c for c in COMMONS if c[0] not in kept]))
    for clip in [*fetched, *free]:
        if clip:
            kept[clip["name"]] = clip
    missing = [case[0] for case in [*CASES, *COMMONS] if case[0] not in kept]
    lock.write_text(json.dumps(list(kept.values()), indent=2) + "\n", encoding="utf-8")
    print(f"{len(kept)} clip(s) in the lock file; not found: {missing or 'none'}")


if __name__ == "__main__":
    main()
