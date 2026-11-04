# crisis-moments

Finds moments in a video where a person may be in distress or in conflict, for a person to look at: screaming or shouting, crying, struggling to breathe, the sounds of a blow, words of distress or about harming oneself, and aggressive words. A Metil plugin written in Rust. It uses fixed rules on what other steps found. It has no model, needs no network and downloads nothing.

**What it is not.** It does not decide that a crisis, a panic attack or an attack happened, and it does not judge anyone. It never writes those words: it makes a short list of "moments to review" and says what was heard (`screaming`, `crying`, `words of distress`...). It will flag moments that are fine (films, music, sport, games, a loud laugh) and miss some that are not, and it cannot see anything: a fight in complete silence is invisible to it. It is not a medical device or a safety system. Do not use it to decide anything about a person.

**What it does not look for:** alarms, sirens, gunshots, explosions and other sounds of danger that are not a person. They are clips it must leave alone.

## Action `find`

Give at least one of the three lists. A list that is given but empty is fine: it means there was nothing to see, and the result is empty. Giving none of them is a 400 failure.

| Input | Shared type | Where it comes from |
|---|---|---|
| `text` | `timedText` | the `sentences` of `speech-to-text.transcribe` |
| `labels` | `timedLabel` | the `events` of `sound-events.find` |
| `volume` | `series` | the `windows` of `sound-volume.map` (`value` is the level in dB) |

Loudness only **supports** the other two: a window much louder than usual raises the confidence of a sound or words at the same time, but it never makes a moment alone (a door, thunder and a shout all look the same to it).

| Setting | |
|---|---|
| `languages` | the built-in lists of words to use: `en` English, `pt` Portuguese, `es` Spanish, `fr` French, `de` German (default `["en"]`). **Give every language the video has**: the plugin cannot tell which language a sentence is in, and a language that is not listed finds nothing. The lists were written by a language model and have not been checked by a native speaker. |
| `distressWords` | words or phrases of a person in distress (a cry for help, not being able to breathe, fear, "stop"). A list you give replaces the built-in lists of this kind, whatever `languages` says. |
| `selfHarmWords` | words or phrases about ending one's own life or hurting oneself. One of them is enough to make a moment. Replaces the built-in lists of this kind. |
| `aggressionWords` | threats and insults. Replaces the built-in lists of this kind. |
| `soundNames` | parts of sound names that flag a found sound, ignoring case (default: scream, shout, yell, bellow, cry, sob, whimper, wail, moan, gasp, pant, wheeze, slap, smack, thump, thud, whack, thwack, slam, grunt, groan). |
| `wordsWindowMs` | words of one kind said within this many milliseconds of the first one count together, even in different sentences (default 10000, 1000 to 600000). A speech recogniser splits "Help! I can't breathe! I'm going to die!" into three sentences; counted together they are three words. |
| `loudnessJumpDb` | how far above the usual level a window must be to support a sound or words, 1 to 60 (default 15). The usual level is the median of all windows. |
| `mergeGapMs` | hints at most this far apart make one moment (default 2000) |
| `minMs` | shorter moments are dropped (default 0) |
| `threshold` | moments less sure than this, 0 to 1, are dropped (default 0.5) |

## Result

```json
{
  "count": 1,
  "moments": [
    {
      "startMs": 10000, "endMs": 12000,
      "label": "screaming", "confidence": 0.95,
      "sources": ["labels", "volume"],
      "evidence": [
        { "source": "labels", "detail": "heard Screaming" },
        { "source": "volume", "detail": "30 dB louder than usual" }
      ]
    }
  ]
}
```

`moments` is a `timedLabel` list: each item has `startMs`, `endMs`, `label` (what was heard) and `confidence`. It can go to any step that reads timed labels. `label` is one of: `screaming`, `shouting`, `crying`, `fast breathing`, `hitting sounds`, `groaning`, `sound of distress`, `words of distress`, `aggressive words`.

## How the rules work

1. Each input gives hints.
   - **Words.** For each kind of words (distress, harming oneself, aggression) the different words said within `wordsWindowMs` make one hint. Its strength: one word 0.45 (one word alone, such as "can you help me carry this", stays under the default threshold), two 0.6, three 0.75, four or more 0.9. For words about harming oneself one phrase is 0.7 and each further one adds 0.1.
   - **Sounds.** A found sound whose label holds one of the sound names gives a hint of the finder's confidence (0.5 if it has none) times a share that depends on how easily other things make that sound: screaming, shouting and crying keep all of it, fast breathing 0.7, hitting sounds 0.6, groaning 0.5.
   - **Loudness.** A window at least `loudnessJumpDb` above the usual level gives a hint that grows to 1 at twice the jump (never below 0.3). It only supports the others.
2. Hints that overlap, or are at most `mergeGapMs` apart, make one moment. A group that holds only loudness is dropped.
3. The confidence of a moment adds up the strongest hint of each input as independent evidence: 1 minus the chance that all are wrong. Two hints from one input do not add up (they are not independent).
4. The label of a moment comes from words or sounds, never from loudness.
5. Moments below `threshold`, or shorter than `minMs`, are dropped.

## What it was tested on

The unit tests use made-up lists for each rule. On top of that the whole chain was measured: `speech-to-text` (tiny model), `sound-events` and `sound-volume` feeding `crisis-moments`, through `metil run`, on 25 clips, with all five languages on. The clips, where each came from and its licence are in `evaluation/clips.lock.json`; the media is not kept in the repository. `evaluation/fetch_clips.py` and `evaluation/make_speech_clips.py` get them again, and `evaluation/evaluate.py` repeats the measurement. The flow is [examples/moments-to-review.mtf](examples/moments-to-review.mtf).

**9 clips a person should look at: 7 found.**

| Clip | Found |
|---|---|
| a woman screaming (a recording, Creative Commons) | `screaming` 0.95 |
| a woman sobbing (a sound effect) | `crying` 0.55 |
| an English sentence of distress, one about harming oneself, one of threats (a synthetic voice) | `words of distress` 0.75, `words of distress` 0.80, `aggressive words` 0.75 |
| a Portuguese sentence of distress and one of threats (a synthetic voice) | `words of distress` 0.75, `aggressive words` 0.60 |
| a baby crying with a music bed (a sound effect) | **not found**: `sound-events` hears crying, whimpering and screaming, but at 0.2 to 0.4, too unsure to count |
| a punch (a film-style sound effect) | **not found**: `sound-events` hears a vehicle and a train, so no hit is heard |

**16 clips a person should not look at: none flagged.** An alarm, a siren, gunshots, an explosion and a heartbeat (sounds of danger that are not a person), a football stadium crowd, 1920s piano, a spoken article, a loud brass band, applause, thunder, and five calm sentences: with the word "help" ("can you help me carry the groceries"), a report of a fire to the fire brigade, the weather, and the Portuguese shopping and weather sentences.

**What this does not tell you.** It is a small set of clean sounds and short synthetic sentences: no film, no street noise, no overlapping voices, no real person in distress. It says nothing about how often the plugin is wrong on your material. The misses show the real limit: the plugin is only as good as the labels `sound-events` gives it, and a quiet, unsure or mixed sound passes unseen. A lower `threshold` does not fix that: 0.4 flags the harmless "help" sentence, 0.3 also flags a gunshot read as a hit, and neither finds the baby. The measurement found one design flaw on the way, which is fixed: words were counted one sentence at a time, so a call for help that the speech recogniser split into three sentences was missed. Keywords still match the transcript as written, so a word the speech recognition gets wrong is not found. Measure on your own material before you rely on it.

## Privacy

It reads the words people said and the sounds heard, but only as lists that earlier steps made. It writes nothing to disk and uses no network. The recordings can hold people in bad moments: keep them, and what this plugin says about them, to the people who need them.

## Later (not built)

- **Movement.** A separate plugin that tracks body movements (a blow, a fall) and gives a list this plugin would count as one more kind of evidence next to words and sounds: MetilPlugins issue 53.
- An optional emotion input (a model whose licence must be checked first) and an optional language-model check of the middle-confidence moments only. See MetilPlugins issue 10.
- Lists in more languages, and a check of all the lists by native speakers.
