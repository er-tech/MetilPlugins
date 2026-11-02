"""Makes the spoken clips of the evaluation with the voices of Windows (System.Speech), and adds them to
clips.lock.json. They test the words path of the plugin, which no recording of the other clips does.
The voices are synthetic and the sentences are written here, so there is nothing to attribute; the
files are not kept in the repository. The Portuguese ones need the voice Microsoft Maria. Windows
only. Usage: python make_speech_clips.py <folder>
"""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent

MARIA = "Microsoft Maria Desktop"

#: Name, what is said, whether a person should look at it, and the voice (none: the default one).
SENTENCES = [
    ("spoken-distress", "Help! I can't breathe! I'm going to die!", True, None),
    ("spoken-self-harm", "I can't go on. I just want to die.", True, None),
    ("spoken-aggression", "Shut up! I'll kill you! Get out of here!", True, None),
    ("spoken-chore", "Can you help me carry the groceries to the car? Thank you very much.", False, None),
    ("spoken-weather", "Tomorrow the weather will be sunny with a light wind from the west.", False, None),
    ("spoken-fire-call", "Hello, I want to report a small fire in the kitchen. The firefighters are on the way.", False, None),
    ("falado-socorro", "Socorro! Não consigo respirar! Estou com medo!", True, MARIA),
    ("falado-ameaca", "Cala a boca! Eu vou te matar! Sai daqui!", True, MARIA),
    ("falado-compras", "Você pode me ajudar a carregar as compras até o carro? Muito obrigado.", False, MARIA),
    ("falado-tempo", "Amanhã o tempo será ensolarado, com um vento leve vindo do oeste.", False, MARIA),
]

#: The names of earlier versions of these clips, taken out of the lock file when it is rewritten.
OLD_NAMES = ["spoken-alarm"]


def speak(text: str, target: Path, voice: str | None) -> None:
    """Writes `text` spoken by `voice` (the default voice of Windows when none) to the wave file `target`."""
    select = f"$s.SelectVoice('{voice}');" if voice else ""
    script = (
        "Add-Type -AssemblyName System.Speech;"
        "$s = New-Object System.Speech.Synthesis.SpeechSynthesizer;"
        f"{select}"
        f"$s.SetOutputToWaveFile('{target}');"
        f"$s.Speak('{text.replace(chr(39), chr(39) * 2)}');"
        "$s.Dispose()"
    )
    subprocess.run(["powershell", "-NoProfile", "-Command", script], check=True)


def main() -> None:
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    lock = HERE / "clips.lock.json"
    kept = {clip["name"]: clip for clip in json.loads(lock.read_text(encoding="utf-8"))}
    for old in OLD_NAMES:
        kept.pop(old, None)
    for name, text, flag, voice in SENTENCES:
        target = folder / f"{name}.wav"
        speak(text, target, voice)
        kept[name] = {
            "name": name, "search": text, "shouldFlag": flag, "url": "made by make_speech_clips.py",
            "title": text, "uploader": f"synthetic voice of Windows ({voice or 'default'})",
            "licence": "none (synthetic speech)", "seconds": 8.0, "file": target.name,
        }
    lock.write_text(json.dumps(list(kept.values()), indent=2) + chr(10), encoding="utf-8")
    print(f"{len(kept)} clip(s) in the lock file")


if __name__ == "__main__":
    main()
