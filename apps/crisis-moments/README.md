# crisis-moments

Finds moments in a video that a person should look at: shouting, an alarm, a sudden loud sound, or words of alarm. A Metil plugin written in Rust. It uses fixed rules on what other steps found. It has no model, needs no network and downloads nothing.

**What it is not.** It does not decide that a crisis happened, and it does not judge anyone. It makes a short list of "moments to review" and says why. It will flag moments that are fine (films, music, sport, a loud laugh) and miss some that are not. It is not a medical device or a safety system. Do not use it to decide anything about a person.

## Action `find`

Give at least one of the three lists. A list that is given but empty is fine: it means there was nothing to see, and the result is empty. Giving none of them is a 400 failure.

| Input | Shared type | Where it comes from |
|---|---|---|
| `text` | `timedText` | the `sentences` of `speech-to-text.transcribe` |
| `labels` | `timedLabel` | the `events` of `sound-events.find`, or the `appearances` of `coco-objects.find` |
| `volume` | `series` | the `windows` of `sound-volume.map` (`value` is the level in dB) |

| Setting | |
|---|---|
| `languages` | the built-in lists of words of alarm to use: `en` English, `pt` Portuguese, `es` Spanish, `fr` French, `de` German (default `["en"]`). **Give every language the video has**: the plugin cannot tell which language a sentence is in, and a language that is not listed finds nothing. The lists are written by a language model and have not been checked by a native speaker. |
| `keywords` | words or phrases that flag a sentence, as whole words, ignoring case. A list you give replaces every built-in list, whatever `languages` says. |
| `soundNames` | parts of sound names that flag a found sound, ignoring case (default: scream, shout, yell, cry, sob, siren, alarm, glass, shatter, explosion, gunshot, gunfire, bang) |
| `loudnessJumpDb` | how far above the usual level a window must be, 1 to 60 (default 15). The usual level is the median of all windows. |
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
      "label": "shouting", "confidence": 0.9,
      "sources": ["labels", "volume"],
      "evidence": [
        { "source": "labels", "detail": "heard Screaming" },
        { "source": "volume", "detail": "30 dB louder than usual" }
      ]
    }
  ]
}
```

`moments` is a `timedLabel` list: each item has `startMs`, `endMs`, `label` (why it was flagged) and `confidence`. It can go to any step that reads timed labels. `label` is one of: `shouting`, `crying`, `alarm`, `loud impact`, `alarming sound`, `words of alarm`, `sudden loud sound`.

## How the rules work

1. Each input gives hints. A sentence with keywords gives a hint whose strength grows with the number of different keywords in it: one word 0.45, two 0.6, three 0.75, four or more 0.9. One word alone ("can you help me carry this") stays under the default threshold, so it needs another input to agree or a lower `threshold`. A found sound whose label holds one of the sound names gives a hint of its own confidence (0.5 if it has none). A window at least `loudnessJumpDb` above the usual level gives a hint that grows to 1 at twice the jump (never below 0.3).
2. Hints that overlap, or are at most `mergeGapMs` apart, make one moment.
3. The confidence of a moment adds up the strongest hint of each input as independent evidence: 1 minus the chance that all are wrong. One keyword alone gives 0.45 and is dropped at the default threshold; a keyword plus a scream gives more. Two hints from one input do not add up.
4. The label of a moment comes from words or sounds first, and from loudness only when there is nothing else.
5. Moments below `threshold`, or shorter than `minMs`, are dropped.

## What it was tested on

The unit tests use made-up lists for each rule. On top of that the whole chain was measured: `speech-to-text` (tiny model), `sound-events` and `sound-volume` feeding `crisis-moments`, through `metil run`, on 17 clips. The clips, where each came from and its licence are in `evaluation/clips.lock.json`; the media is not kept in the repository. `evaluation/fetch_clips.py` and `evaluation/make_speech_clips.py` get them again, and `evaluation/evaluate.py` repeats the measurement. The flow is [examples/moments-to-review.mtf](examples/moments-to-review.mtf).

| What a person should look at (7) | Found |
|---|---|
| a fire alarm, an ambulance siren, a scream, gunshots, an explosion (recordings, Creative Commons) | all 5: `alarm`, `alarm`, `shouting`, `loud impact` (twice) |
| an alarm sentence in English, one in Portuguese (a synthetic voice) | both, as `words of alarm` (the Portuguese one only with `pt` in `languages`) |

| What a person should not look at (10) | Found |
|---|---|
| a football stadium crowd, 1920s piano, a spoken article, a loud brass band, applause, thunder (recordings) | nothing |
| a calm sentence with the word "help", a plain sentence, in English and in Portuguese (a synthetic voice) | nothing |

**What this does not tell you.** It is a small set of clean, single sounds and short sentences, not real scenes: no film, no street noise, no overlapping voices, and the speech is synthetic. It says nothing about how often the plugin is wrong on your material. It found one weakness on the way, which is fixed: the word "help" alone in a harmless sentence made a flag, so a single keyword now stays below the default threshold. Keywords still match the transcript as written, so a word the speech recognition gets wrong is not found. Measure on your own material before you rely on it.

## Privacy

It reads the words people said and the sounds heard, but only as lists that earlier steps made. It writes nothing to disk and uses no network.

## Later (not built)

An optional emotion input, a movement input, and an optional language-model check of the middle-confidence moments only. See MetilPlugins issue 10. More languages than the five, and a check of the lists by native speakers.
