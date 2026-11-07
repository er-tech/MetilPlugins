# sentiment-analysis

Says how **positive, neutral or negative** each sentence of a transcript is, with a model that runs on the computer and **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas loads the plugin when a flow calls it; the model is downloaded once, the first time.

It rates the **words**. It does not hear the voice, and it does not know what the words mean to the person who said them: read "Be honest about what it cannot do" below before you build on it.

## `analyze`

| Input | |
|---|---|
| `segments` | the sentences to rate, each with `startMs`, `endMs` and `text`: connect the `sentences` of `speech-to-text` |
| `model` | `standard` (default, about 136 MB to download) or `full` (about 541 MB, a little more sure of itself) |
| `device` | `cpu` (default), `auto` (what Loomas grants), `gpu` or `gpu:N`; see the manual page on the graphics card in the Loomas repository |

It gives:

```json
{
  "segments": [
    { "startMs": 0, "endMs": 2000, "text": "I love this, it is wonderful!", "label": "positive",
      "confidence": 0.91, "score": 0.85, "probabilities": { "positive": 0.88, "neutral": 0.04, "negative": 0.09 } }
  ],
  "overall": { "label": "neutral", "score": 0.246, "segments": 7, "positive": 5, "neutral": 0, "negative": 2 },
  "skipped": 1,
  "model": "standard",
  "device": { "device": "cpu", "name": null, "fallback": null }
}
```

- `label` is the most likely of `positive`, `neutral` and `negative`. `confidence` is how likely that label is. `score` is the chance of positive minus the chance of negative, from -1 to 1, so a flow can use a number.
- `segments` is typed as timed labels (`label` is there), so it goes into `data.mergeEvents` as one more source next to the sounds and the words.
- `overall` is one value for the whole transcript: the mean score weighted by how long each sentence lasts, and the label it comes to (above 0.25 positive, below -0.25 negative, else neutral), with the number of sentences of each label. `score` is `null` when there was nothing to rate.
- A sentence with no words is left out and counted in `skipped`. No sentences is a success with an empty list.
- Times are as they were given. The plugin never changes them.

## The model

[`lxyuan/distilbert-base-multilingual-cased-sentiments-student`](https://huggingface.co/lxyuan/distilbert-base-multilingual-cased-sentiments-student) (**Apache-2.0**): a small DistilBERT that learned from a larger model to tell the three apart in 12 languages: English, Portuguese, Spanish, French, German, Italian, Arabic, Hindi, Indonesian, Malay, Japanese and Chinese. It is used as Xenova converted it to ONNX (`standard` is the 8-bit version, `full` the one with 32-bit numbers). Both files are checked against a sum before they are used. The ONNX file in the repository of the author gives the answer "negative" for every text, so it is not used.

## Be honest about what it cannot do

- **It rates wording, not distress.** On the test below "Me ajuda, por favor, estou com muito medo!" ("Help me, please, I am very afraid!") is rated **positive** (0.90): the polite words weigh more than the fear. A sentiment model is not a way to find a person in danger. The `crisis-moments` plugin does not use it for that.
- **Sarcasm and irony are not understood.** "Great, another Monday" reads as positive.
- **It was trained on short texts** (reviews and posts). Spoken language, with its pauses and its mistakes in the transcript, is harder, and the transcript's own mistakes pass straight into the rating.
- **Neutral is weak.** A plain statement ("The table is in the room") gets a low `confidence` (0.44) and may come out positive. Treat a low `confidence` as "no opinion".
- **The model rates one sentence at a time.** It does not know what was said before. A text of more than 512 pieces is cut.

Tested on seven sentences (English and Portuguese) on a Windows computer: the clear ones were right (positive 0.91 and 0.97, negative 0.99), the fear sentence and the plain statement were not, as said above. The seven sentences took 7.5 s the first time on the processor, including loading the model. This is a small test, not a measurement of accuracy. It was not run on a graphics card.

## Failures

| Code | When |
|---|---|
| 400 | a param is wrong, or a sentence has no time or no text |
| 422 | the model or its tokenizer could not be read |
| 503 | the model could not be downloaded (no network the first time) |

## Install

Nothing to install. The first call downloads the model into `~/.loomas/models/sentiment-analysis` (or the folder `LOOMAS_MODELS` names) and checks its sum.
