using MyAiUsage.Core;

namespace MyAiUsage.App;

internal static class RuntimeStatus
{
    internal static string Partial(bool hasSnapshot) =>
        hasSnapshot ? "Dados parciais — Desatualizado" : "Dados parciais";

    internal static string ForError(CodexClientErrorKind kind) => kind switch
    {
        CodexClientErrorKind.ExecutableNotFound => "Codex ausente — instale o Codex e adicione-o ao PATH",
        CodexClientErrorKind.AuthenticationRequired => "Desconectado — execute codex login",
        CodexClientErrorKind.Timeout => "Tempo esgotado — verifique a conexão e tente novamente",
        CodexClientErrorKind.PartialData => "Dados parciais",
        CodexClientErrorKind.Cancelled => "Atualização cancelada",
        _ => "Falha temporária — tente novamente"
    };
}
