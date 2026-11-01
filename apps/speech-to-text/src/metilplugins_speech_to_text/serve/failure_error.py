"""A call that failed, as the plugin decides."""

from typing import Any


class Failure(Exception):  # noqa: N818 - named as in the protocol and the other SDKs
    """Raise it to answer a call with an `error`. Only the message is required.

    `code` is a number with the meaning of the HTTP status code of that number (400, 404, 503...)
    or a name of the plugin's own (`MODEL_UNAVAILABLE`); the host knows what the standard numbers
    mean, for instance whether a retry could work. `retryable` is only for when the plugin knows
    better. `details` is anything else worth keeping.
    """

    def __init__(
        self,
        message: str,
        code: int | str | None = None,
        retryable: bool | None = None,
        details: Any = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.code = code
        self.retryable = retryable
        self.details = details

    @classmethod
    def status(cls, code: int, message: str) -> "Failure":
        """A failure with the meaning of an HTTP status code."""
        return cls(message, code=code)

    def to_message(self, call_id: Any) -> dict[str, Any]:
        """The `error` message for the call with this id, with only what was given."""
        reply: dict[str, Any] = {"type": "error", "id": call_id, "message": self.message}
        if self.code is not None:
            reply["code"] = self.code
        if self.retryable is not None:
            reply["retryable"] = self.retryable
        if self.details is not None:
            reply["details"] = self.details
        return reply
