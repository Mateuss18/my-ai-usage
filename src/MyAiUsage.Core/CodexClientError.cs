namespace MyAiUsage.Core;

public enum CodexClientErrorKind
{
    ExecutableNotFound,
    AuthenticationRequired,
    EndOfStream,
    InvalidJson,
    Timeout,
    ProtocolError,
    PartialData,
    Cancelled
}

public sealed class CodexClientException : Exception
{
    public CodexClientErrorKind Kind { get; }

    public CodexClientException(CodexClientErrorKind kind, string message, Exception? inner = null)
        : base(message, inner) => Kind = kind;
}
