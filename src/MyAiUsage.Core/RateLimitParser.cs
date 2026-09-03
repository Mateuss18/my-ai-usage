using System.Text.Json;

namespace MyAiUsage.Core;

public static class RateLimitParser
{
    private const string PartialDataMessage = "Não foi possível ler as janelas de quota do Codex.";

    public static RateLimitSnapshot Parse(JsonElement result, TimeZoneInfo timeZone)
    {
        var buckets = new List<RateLimitBucket>();
        var isPartial = false;

        if (TryGetObject(result, "rateLimitsByLimitId", out var byLimitId)
            && byLimitId.EnumerateObject().MoveNext())
        {
            foreach (var property in byLimitId.EnumerateObject())
            {
                if (TryParseBucket(property.Name, property.Value, property.Name, timeZone, ref isPartial, out var bucket))
                {
                    buckets.Add(bucket);
                }
                else
                {
                    isPartial = true;
                }
            }
        }
        else if (TryGetObject(result, "rateLimits", out var fallback))
        {
            AddFallbackBuckets(buckets, fallback, timeZone, ref isPartial);
        }

        if (buckets.Count == 0)
        {
            throw new CodexClientException(CodexClientErrorKind.PartialData, PartialDataMessage);
        }

        return new RateLimitSnapshot(buckets, DateTimeOffset.UtcNow, isPartial);
    }

    private static void AddFallbackBuckets(
        List<RateLimitBucket> buckets,
        JsonElement fallback,
        TimeZoneInfo timeZone,
        ref bool isPartial)
    {
        if (TryParseBucket("codex", fallback, "Codex", timeZone, ref isPartial, out var directBucket))
        {
            buckets.Add(directBucket);
            return;
        }

        if (fallback.ValueKind != JsonValueKind.Object)
        {
            isPartial = true;
            return;
        }

        foreach (var property in fallback.EnumerateObject())
        {
            if (TryParseBucket(property.Name, property.Value, property.Name, timeZone, ref isPartial, out var bucket))
            {
                buckets.Add(bucket);
            }
            else
            {
                isPartial = true;
            }
        }
    }

    private static bool TryParseBucket(
        string id,
        JsonElement value,
        string defaultDisplayName,
        TimeZoneInfo timeZone,
        ref bool isPartial,
        out RateLimitBucket bucket)
    {
        bucket = null!;
        if (value.ValueKind != JsonValueKind.Object)
        {
            return false;
        }

        var windows = new List<RateLimitWindow>();
        foreach (var property in value.EnumerateObject())
        {
            if (property.Value.ValueKind != JsonValueKind.Object || !ContainsWindowField(property.Value))
            {
                continue;
            }

            windows.Add(ParseWindow(property.Name, property.Value, timeZone, ref isPartial));
        }

        if (windows.Count == 0)
        {
            return false;
        }

        bucket = new RateLimitBucket(id, GetDisplayName(value, defaultDisplayName), windows);
        return true;
    }

    private static RateLimitWindow ParseWindow(
        string key,
        JsonElement value,
        TimeZoneInfo timeZone,
        ref bool isPartial)
    {
        var usedPercent = ReadUsedPercent(value, ref isPartial);
        var duration = ReadDuration(value, ref isPartial);
        var resetsAt = ReadReset(value, timeZone, ref isPartial);
        return new RateLimitWindow(key, usedPercent, duration, resetsAt);
    }

    private static int? ReadUsedPercent(JsonElement value, ref bool isPartial)
    {
        if (value.TryGetProperty("usedPercent", out var property)
            && property.ValueKind == JsonValueKind.Number
            && property.TryGetInt32(out var used)
            && used is >= 0 and <= 100)
        {
            return used;
        }

        isPartial = true;
        return null;
    }

    private static long? ReadDuration(JsonElement value, ref bool isPartial)
    {
        if (value.TryGetProperty("windowDurationMins", out var property)
            && property.ValueKind == JsonValueKind.Number
            && property.TryGetInt64(out var duration)
            && duration > 0)
        {
            return duration;
        }

        isPartial = true;
        return null;
    }

    private static DateTimeOffset? ReadReset(JsonElement value, TimeZoneInfo timeZone, ref bool isPartial)
    {
        if (value.TryGetProperty("resetsAt", out var property)
            && property.ValueKind == JsonValueKind.Number
            && property.TryGetInt64(out var timestamp)
            && timestamp is >= -62135596800 and <= 253402300799)
        {
            return TimeZoneInfo.ConvertTime(DateTimeOffset.FromUnixTimeSeconds(timestamp), timeZone);
        }

        isPartial = true;
        return null;
    }

    private static bool ContainsWindowField(JsonElement value) =>
        value.TryGetProperty("usedPercent", out _)
        || value.TryGetProperty("windowDurationMins", out _)
        || value.TryGetProperty("resetsAt", out _);

    private static string GetDisplayName(JsonElement value, string fallback)
    {
        foreach (var propertyName in new[] { "limitName", "limitId" })
        {
            if (value.TryGetProperty(propertyName, out var property)
                && property.ValueKind == JsonValueKind.String)
            {
                return property.GetString() ?? fallback;
            }
        }

        return fallback;
    }

    private static bool TryGetObject(JsonElement value, string propertyName, out JsonElement property)
    {
        if (value.ValueKind == JsonValueKind.Object
            && value.TryGetProperty(propertyName, out property)
            && property.ValueKind == JsonValueKind.Object)
        {
            return true;
        }

        property = default;
        return false;
    }
}
