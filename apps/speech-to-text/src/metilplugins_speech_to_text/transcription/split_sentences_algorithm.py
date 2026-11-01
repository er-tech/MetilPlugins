"""Groups the words of a speech into sentences, each with its own start and end time."""

from collections.abc import Sequence

from .sentence_contract import Sentence, TimedWord, Word

#: What ends a sentence when a word finishes with it (after any closing quote or bracket).
_SENTENCE_ENDS = (".", "!", "?", "…", "。", "！", "？")
_CLOSERS = "\"'”’»)]}"

#: Words that end with a full stop without ending the sentence ("Dr. Smith", "e.g. this").
_ABBREVIATIONS = frozenset(
    {
        "mr.", "mrs.", "ms.", "dr.", "prof.", "sr.", "jr.", "st.", "mt.", "vs.", "etc.",
        "e.g.", "i.e.", "no.", "approx.", "inc.", "ltd.", "co.", "u.s.", "u.k.",
    }
)  # fmt: skip


def split_sentences(
    words: Sequence[Word],
    max_gap_seconds: float = 2.0,
    max_words: int = 60,
) -> list[Sentence]:
    """Splits the words into sentences.

    A sentence ends after a word that ends with `.`, `!`, `?` or `…` (not an abbreviation such as
    "Dr."), when the next word comes more than `max_gap_seconds` later (a pause long enough to be a
    new thought, which also covers speech the recognition did not punctuate), or after `max_words`
    words, so a long run without punctuation does not become one enormous sentence.
    """
    sentences: list[Sentence] = []
    current: list[Word] = []

    for word in words:
        if current and (
            word.start - current[-1].end > max_gap_seconds or len(current) >= max_words
        ):
            sentences.append(_close(current))
            current = []
        current.append(word)
        if _ends_a_sentence(word.text):
            sentences.append(_close(current))
            current = []

    if current:
        sentences.append(_close(current))
    return [sentence for sentence in sentences if sentence.text]


def _ends_a_sentence(text: str) -> bool:
    written = text.strip().rstrip(_CLOSERS)
    if not written.endswith(_SENTENCE_ENDS):
        return False
    return written.lower() not in _ABBREVIATIONS


def _close(words: Sequence[Word]) -> Sentence:
    text = "".join(word.text for word in words).strip()
    confidence = sum(word.probability for word in words) / len(words)
    return Sentence(
        start_ms=round(words[0].start * 1000),
        end_ms=round(words[-1].end * 1000),
        text=text,
        confidence=round(confidence, 3),
        words=tuple(
            TimedWord(
                start_ms=round(word.start * 1000),
                end_ms=round(word.end * 1000),
                text=word.text.strip(),
                confidence=round(word.probability, 3),
            )
            for word in words
            if word.text.strip()
        ),
    )
