"""The speech-to-text plugin for Loomas: the sentences said in a video or a sound, with the time
each starts and ends. The speech recognition is faster-whisper; the protocol is spoken by the
`serve` slice, the work is in `transcription`, and `actions` reads the params of a call."""
