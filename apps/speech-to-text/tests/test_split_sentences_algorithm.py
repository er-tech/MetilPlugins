from loomasplugins_speech_to_text.transcription import Word, split_sentences


def words(*items: tuple[float, float, str], probability: float = 0.9) -> list[Word]:
    return [Word(start, end, text, probability) for start, end, text in items]


def test_ends_a_sentence_at_a_full_stop_a_question_mark_and_an_exclamation_mark() -> None:
    heard = words(
        (0.0, 0.4, " Hello"),
        (0.4, 0.9, " there."),
        (1.0, 1.3, " How"),
        (1.3, 1.6, " are"),
        (1.6, 2.0, " you?"),
        (2.1, 2.5, " Great!"),
    )
    sentences = split_sentences(heard)
    assert [sentence.text for sentence in sentences] == ["Hello there.", "How are you?", "Great!"]
    assert [(s.start_ms, s.end_ms) for s in sentences] == [(0, 900), (1000, 2000), (2100, 2500)]


def test_an_abbreviation_does_not_end_a_sentence_and_a_closing_quote_does_not_hide_the_end() -> (
    None
):
    heard = words(
        (0.0, 0.3, " Dr."),
        (0.3, 0.7, " Smith"),
        (0.7, 1.1, " said"),
        (1.1, 1.5, ' "stop."'),
        (1.6, 1.9, " Then"),
        (1.9, 2.4, " e.g."),
        (2.4, 2.8, " this"),
        (2.8, 3.1, " ended…"),
    )
    texts = [sentence.text for sentence in split_sentences(heard)]
    assert texts == ['Dr. Smith said "stop."', "Then e.g. this ended…"]


def test_a_long_pause_starts_a_new_sentence_even_without_punctuation() -> None:
    heard = words((0.0, 0.5, " one"), (0.5, 1.0, " two"), (4.0, 4.5, " three"), (4.5, 5.0, " four"))
    sentences = split_sentences(heard, max_gap_seconds=2.0)
    assert [sentence.text for sentence in sentences] == ["one two", "three four"]
    assert [sentence.start_ms for sentence in sentences] == [0, 4000]
    # With a bigger allowance the same words are one sentence.
    assert len(split_sentences(heard, max_gap_seconds=5.0)) == 1


def test_a_long_run_without_punctuation_is_cut_after_the_most_words_allowed() -> None:
    heard = words(*[(n * 0.3, n * 0.3 + 0.25, f" w{n}") for n in range(25)])
    sentences = split_sentences(heard, max_words=10)
    assert [len(sentence.text.split()) for sentence in sentences] == [10, 10, 5]


def test_the_confidence_is_the_average_of_its_words_and_times_are_whole_milliseconds() -> None:
    heard = [Word(0.0004, 0.5004, " Hi", 0.5), Word(0.5004, 1.0006, " you.", 1.0)]
    (sentence,) = split_sentences(heard)
    assert sentence.confidence == 0.75
    assert (sentence.start_ms, sentence.end_ms) == (0, 1001)
    assert sentence.to_dict() == {
        "startMs": 0,
        "endMs": 1001,
        "text": "Hi you.",
        "confidence": 0.75,
    }


def test_no_words_give_no_sentences_and_a_number_with_a_point_is_not_the_end() -> None:
    assert split_sentences([]) == []
    heard = words((0.0, 0.3, " Costs"), (0.3, 0.6, " 3.50"), (0.6, 0.9, " euros."))
    assert [sentence.text for sentence in split_sentences(heard)] == ["Costs 3.50 euros."]


def test_other_scripts_and_their_full_stops_work_too() -> None:
    heard = words((0.0, 0.5, "今日は。"), (0.6, 1.0, "元気ですか？"), (1.1, 1.5, " Olá!"))
    assert [sentence.text for sentence in split_sentences(heard)] == [
        "今日は。",
        "元気ですか？",
        "Olá!",
    ]
