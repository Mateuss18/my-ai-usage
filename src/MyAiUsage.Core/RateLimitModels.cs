namespace MyAiUsage.Core;

public sealed record RateLimitWindow(
    string Key,
    int? UsedPercent,
    long? WindowDurationMins,
    DateTimeOffset? ResetsAt);

public sealed record RateLimitBucket(
    string Id,
    string DisplayName,
    IReadOnlyList<RateLimitWindow> Windows);

public sealed record RateLimitSnapshot(
    IReadOnlyList<RateLimitBucket> Buckets,
    DateTimeOffset RetrievedAt,
    bool IsPartial);

public static class QuotaPresentation
{
    public static string WindowTitle(long? minutes) => minutes switch
    {
        null or <= 0 => "Janela sem duração informada",
        var value when value % 1440 == 0 => $"Janela de {value / 1440} dias",
        var value when value % 60 == 0 => $"Janela de {value / 60} horas",
        var value => $"Janela de {value} minutos"
    };

    public static string UsageColor(int? usedPercent) => usedPercent switch
    {
        >= 0 and < 50 => "green",
        >= 50 and < 80 => "yellow",
        >= 80 and <= 100 => "red",
        _ => "neutral"
    };

    public static string UsageState(int? usedPercent) => usedPercent switch
    {
        100 => "Limite atingido",
        >= 0 and <= 100 => "Disponível",
        _ => "Uso desconhecido"
    };
}
