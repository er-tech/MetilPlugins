"""Makes the spoken clips of the evaluation with the voice of Windows (System.Speech), and adds them to
clips.lock.json. They test the words path of the plugin, which no recording of the other clips does.
The voice is synthetic and the sentences are written here, so there is nothing to attribute; the
files are not kept in the repository. Windows only. Usage: python make_speech_clips.py <folder>
"""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent

#: Name, what is said, whether a person should look at it, and the voice (none: the default one).
SENTENCES = [
    ("spoken-alarm", "Help! Somebody call the police! There is a fire and he is bleeding!", True, None),
    ("spoken-chore", "Can you help me carry the groceries to the car? Thank you very much.", False, None),
    ("spoken-weather", "Tomorrow the weather will be sunny with a light wind from the west.", False, None),
    ("falado-socorro", "Socorro! Chamem a polícia! Tem um incêndio e ele está sangrando!", True, "Microsoft Maria Desktop"),
    ("falado-compras", "Você pode me ajudar a carregar as compras até o carro? Muito obrigado.", False, "Microsoft Maria Desktop"),
    ("falado-tempo", "Amanhã o tempo será ensolarado, com um vento leve vindo do oeste.", False, "Microsoft Maria Desktop"),
]


def speak(text: str, target: Path, voice: str | None) -> None:
    """Writes `text` spoken by `voice` (the default voice of Windows when none) to the wave file `target`."""
    select = f"$s.SelectVoice('{voice}');" if voice else ""
    script = (
        "Add-Type -AssemblyName System.Speech;"
        "$s = New-Object System.Speech.Synthesis.SpeechSynthesizer;"
        f"{select}"
        f"$s.SetOutputToWaveFile('{target}');"
        f"$s.Speak('{text}');"
        "$s.Dispose()"
    )
    subprocess.run(["powershell", "-NoProfile", "-Command", script], check=True)


def main() -> None:
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    lock = HERE / "clips.lock.json"
    kept = {clip["name"]: clip for clip in json.loads(lock.read_text(encoding="utf-8"))}
    for name, text, flag, voice in SENTENCES:
        target = folder / f"{name}.wav"
        speak(text, target, voice)
        kept[name] = {
            "name": name, "search": text, "shouldFlag": flag, "url": "made by make_speech_clips.py",
            "title": text, "uploader": f"synthetic voice of Windows ({voice or 'default'})", "licence": "none (synthetic speech)",
            "seconds": 8.0, "file": target.name,
        }
    lock.write_text(json.dumps(list(kept.values()), indent=2) + chr(10), encoding="utf-8")
    print(f"{len(kept)} clip(s) in the lock file")


if __name__ == "__main__":
    main()
