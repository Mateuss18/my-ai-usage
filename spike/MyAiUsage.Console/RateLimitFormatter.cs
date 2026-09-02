using System.Text.Json;

namespace MyAiUsage;

public static class RateLimitFormatter
{
    public static IReadOnlyList<string> Format(JsonElement result, TimeZoneInfo timeZone)
    {
        var lines = new List<string>();

        if (result.TryGetProperty("rateLimitsByLimitId", out var buckets)
            && buckets.ValueKind == JsonValueKind.Object)
        {
            foreach (var bucket in buckets.EnumerateObject())
            {
                AddWindows(lines, bucket.Value, GetLabel(bucket.Value) ?? bucket.Name, timeZone);
            }
        }

        if (lines.Count == 0
            && result.TryGetProperty("rateLimits", out var fallback)
            && fallback.ValueKind == JsonValueKind.Object)
        {
            AddWindows(lines, fallback, GetLabel(fallback) ?? "Codex", timeZone);
        }

        if (lines.Count == 0)
        {
            lines.Add("Quotas indisponíveis.");
        }

        return lines;
    }

    private static void AddWindows(
        List<string> lines,
        JsonElement bucket,
        string label,
        TimeZoneInfo timeZone
    )
    {
        AddWindow(lines, bucket, "primary", "primária", label, timeZone);
        AddWindow(lines, bucket, "secondary", "secundária", label, timeZone);
    }

    private static void AddWindow(
        List<string> lines,
        JsonElement bucket,
        string propertyName,
        string windowName,
        string label,
        TimeZoneInfo timeZone
    )
    {
        if (!bucket.TryGetProperty(propertyName, out var window)
            || window.ValueKind != JsonValueKind.Object)
        {
            return;
        }

        var usage = window.TryGetProperty("usedPercent", out var usedPercent)
            && usedPercent.ValueKind == JsonValueKind.Number
            && usedPercent.TryGetInt32(out var used)
            && used is >= 0 and <= 100
                ? $"{used}% usado"
                : "uso desconhecido";
        var duration = window.TryGetProperty("windowDurationMins", out var durationMins)
            && durationMins.ValueKind == JsonValueKind.Number
            && durationMins.TryGetInt64(out var minutes)
                ? $"{minutes} min"
                : "duração desconhecida";
        var reset = window.TryGetProperty("resetsAt", out var resetsAt)
            && resetsAt.ValueKind == JsonValueKind.Number
            && resetsAt.TryGetInt64(out var timestamp)
            && timestamp is >= -62135596800 and <= 253402300799
                ? $"reset {TimeZoneInfo.ConvertTime(DateTimeOffset.FromUnixTimeSeconds(timestamp), timeZone):yyyy-MM-dd HH:mm zzz}"
                : "reset desconhecido";
        lines.Add($"{label} — {windowName}: {usage} | {duration} | {reset}");
    }

    private static string? GetLabel(JsonElement bucket)
    {
        foreach (var propertyName in new[] { "limitName", "limitId" })
        {
            if (bucket.TryGetProperty(propertyName, out var value)
                && value.ValueKind == JsonValueKind.String)
            {
                return value.GetString();
            }
        }

        return null;
    }
}
