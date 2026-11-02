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
CASES = [
    ("alarm", "fire alarm sound effect", True),
    ("siren", "ambulance siren sound", True),
    ("scream", "woman screaming sound effect", True),
    ("crying", "baby crying", True),
    ("gunshot", "gunshot sound effect", True),
    ("explosion", "explosion sound effect", True),
    ("help-words", "short film actor shouting help emergency", True),
    ("talk", "interview speaking calmly", False),
    ("piano", "piano classical music", False),
    ("rock", "rock band live music", False),
    ("applause", "crowd applause cheering", False),
    ("laughter", "laughing laughter", False),
    ("rain", "rain forest ambience", False),
    ("guitar", "acoustic guitar playing", False),
    ("stadium", "football stadium crowd", False),
    ("ocean", "ocean waves sound", False),
    ("lecture", "lecture presentation talk", False),
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
            "--print", "after_move:%(id)s|%(duration)s|%(license)s|%(uploader)s|%(title)s|%(filepath)s",
            f"ytsearch40:{term}",
        ],
        capture_output=True, text=True, timeout=900, check=False,
    )
    for line in done.stdout.splitlines():
        parts = line.split("|")
        if len(parts) == 6 and len(parts[0]) == 11:
            video, duration, licence, uploader, title, path = parts
            return {
                "name": name, "search": term, "shouldFlag": flag,
                "url": f"https://www.youtube.com/watch?v={video}", "title": title,
                "uploader": uploader, "licence": licence, "seconds": float(duration),
                "file": Path(path).name,
            }
    return None


def fetch_commons(case: tuple[str, str, bool], folder: Path) -> dict | None:
    """Downloads one file of Wikimedia Commons, with its licence; `None` when it cannot be fetched."""
    name, title, flag = case
    headers = {"User-Agent": "metil-crisis-eval/0.1 (https://github.com/er-tech/MetilPlugins)"}
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


def main() -> None:
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    lock = HERE / "clips.lock.json"
    kept = {clip["name"]: clip for clip in json.loads(lock.read_text(encoding="utf-8"))} if lock.exists() else {}
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
