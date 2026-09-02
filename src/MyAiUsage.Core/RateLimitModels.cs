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
