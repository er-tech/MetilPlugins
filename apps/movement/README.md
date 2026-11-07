# movement

Lists the **movements of the people in a video**: when each starts and ends, what was seen, and which person. It follows the bodies of the people (the keypoints: shoulders, elbows, wrists, hips, knees, ankles), measures how fast the hands and feet move compared with the body, and turns that into a short list of movements. A Loomas plugin written in Rust: one program, nothing to install. The models are downloaded once, the first time.

It sees **bodies, never faces or names**, and it says **what moved, never what it means**. A fast arm is not an attack, and a person on the ground is not hurt. It was asked for by the [crisis-moments](../crisis-moments) work, as one more kind of evidence next to the words and the sounds, and it is useful by itself.

## `find`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it. Loomas decodes five colour pictures a second of it, 480 pixels wide |
| `model` | how the bodies are read: `rtmpose-small` (default), `rtmpose-medium` or `movenet`; see [Choosing a model](#choosing-a-model) |
| `picturesPerSecond` | how many of the five pictures a second are looked at: `5` (default), `2.5` or `1`. More than 5 is not offered, because the number of pictures is set in the manifest, by what Loomas decodes for this plugin |
| `armSpeed`, `legSpeed` | how fast a wrist or an ankle must move to count, in torso lengths a second compared with the body (defaults 4 and 5) |
| `minConfidence` | movements rated below this are left out (default 0.3) |
| `gapMs` | movements of the same kind by the same person no further apart than this are one (default 1000) |
| `minSamples` | a movement seen in fewer pictures than this is left out (default 1; raise it to ignore one-picture blips) |
| `minPersonScore` | how sure the model must be that it is a person (default 0.5; MoveNet is held to at most 0.25, because it rates people lower) |
| `maxPeople` | no more people than this are followed in a picture, the surest first (default 8) |
| `device` | `auto` (default: what Loomas grants), `cpu`, `gpu` or `gpu:N` |

It gives:

```json
{
  "movements": [
    { "startMs": 12000, "endMs": 12400, "label": "fast arm movement", "confidence": 0.62, "person": 2,
      "detail": "right wrist 5.1 torso lengths a second" }
  ],
  "people": 2, "samples": 150, "durationMs": 30000,
  "model": "rtmpose-small",
  "device": { "device": "directml:0", "name": "AMD Radeon RX 7900 XT", "fallback": null }
}
```

`movements` is typed as timed labels, so it goes into `data.mergeEvents` as one more source, and into `crisis-moments`. `person` is a number for this run only: the same person keeps it while they are followed, and it is never an identity. A video with nobody in it is a success with no movements.

## The vocabulary

| Label | What is measured |
|---|---|
| `fast arm movement` | a wrist moves faster than `armSpeed` torso lengths a second, compared with the middle of the body |
| `fast leg movement` | an ankle moves faster than `legSpeed`, compared with the middle of the body |
| `striking motion towards another person` | a fast wrist whose direction is within about 45 degrees of the way to another person, and that ends within one and a half torso lengths of them |
| `falling` | the hips drop 0.8 torso lengths or more within one second while the torso ends up leaning 45 degrees or more |
| `person on the ground` | the torso leans 60 degrees or more and the box is wider than 0.8 of its height |
| `two people close with fast movement` | a fast arm or leg while the middles of two boxes are no further apart than 0.9 of the taller box |

Speeds are in **torso lengths a second** (shoulders to hips, the middle value over the track), so a person far away and a person close by are judged alike. They are measured **compared with the middle of the body**, so a person who walks or runs across the picture with still arms is not taken for one who swings them. `confidence` is half the score of the keypoints plus up to another half for how far past its limit the movement went, all times the score of the keypoints. It rates the measurement. It is not a chance that something happened.

## Choosing a model

The three models read bodies in different ways. Pick by what you film and how much time you have.

| Model | Download | How it works | Good at | Weak at |
|---|---|---|---|---|
| `rtmpose-small` (default) | about 39 MB | a detector finds the people, then a pose network reads each one in a crop | people far away, small or partly hidden; many people (no limit but `maxPeople`); the best balance | a little less exact on the joints than the medium one |
| `rtmpose-medium` | about 145 MB | the same with larger networks | the most exact keypoints; finds more of the small and distant people | the slowest on the processor (about 3 times the small one); a bigger download |
| `movenet` | about 19 MB | one small network reads the whole picture | the fastest, the smallest, nothing else to load | finds **six people at most**, and misses people who are small in the picture or who overlap, which are the cases that matter most here; it rates people low, so its limit is capped at 0.25 |

Measured on this computer (a Windows PC with an RX 7900 XT), 20 pictures of 640 by 360, including starting the program and decoding, so use the numbers for the order and not for the size:

| Model | Processor | Graphics card |
|---|---|---|
| `rtmpose-small` | 1.2 s | 1.3 s |
| `rtmpose-medium` | 3.4 s | 1.7 s |
| `movenet` | 0.9 s | 1.1 s |

Counting people in three photos (a skier alone; a room with a person far away; a street scene with about ten people): `rtmpose-small` found 1, 1 and 8; `rtmpose-medium` found 1, 2 and 8 (the limit is `maxPeople`, 8); `movenet` found 1, 0 and 6 (the most it can). This is a look at three photos, not a benchmark.

The graphics card gives the medium model about twice the speed and the small ones nothing at this size: the networks are small, and sending each crop to the card costs as much as it saves. `device: cpu` is a good choice for the small model.

## Be honest about what it cannot do

- **It was not tested on real footage of fights, falls or struggles.** The limits (`armSpeed` 4, `legSpeed` 5, a fall of 0.8 torso lengths, a lean of 45 and 60 degrees) are reasoned guesses. The unit tests use made-up tracks of keypoints (a punch, a fall, a person lying, a person walking past), and the three models were run on still photos and on a pan across a still photo, where it correctly found **no** movement. Nobody has checked how often it is wrong on a real video. Try it on your own videos and change the limits.
- **Five pictures a second** catches a strike or a fall that lasts a quarter of a second, and misses a quicker one.
- **A wrong keypoint looks like a fast movement.** One picture where an elbow jumps gives one blip. `minSamples` 2 removes it, and costs the quickest real movements.
- **Overlapping or hidden people** lose their keypoints, and two people who cross can swap their numbers.
- **A cut in the video** ends the tracks, and the people after it are new people.
- **It is not a way to find a person in danger.** Dancing, sport, play, a hug and a trip on the stairs make the same lists as the things it was asked for. It gives measurements, and a person has to look.

## The models

| File | Licence | From |
|---|---|---|
| YOLOX (tiny and medium), trained to find people | Apache-2.0 | OpenMMLab, as an ONNX file in a zip |
| RTMPose (small and medium, trained on seven body datasets) | Apache-2.0 | OpenMMLab, as an ONNX file in a zip |
| MoveNet multipose lightning | Apache-2.0 | Google, as ONNX that Xenova converted |

Each file is checked against a sum before it is used. Ultralytics YOLO pose (AGPL-3.0) and OpenPose (a licence for non-commercial use) are not used. MediaPipe BlazePose was not added. ONNX Runtime has no binary for Intel Macs, so the plugin runs on Windows, on Linux (x86-64 and arm64) and on Macs with Apple silicon.

## Failures

| Code | When |
|---|---|
| 400 | a param is wrong, or the media is not a video Loomas decoded for this plugin |
| 422 | a network could not be read, or a model file could not be unpacked |
| 503 | a model could not be downloaded (no network the first time) |

## Install

Nothing to install. The first call downloads the model it needs into `~/.loomas/models/movement` (or the folder `LOOMAS_MODELS` names), checks its sum and unpacks it.
