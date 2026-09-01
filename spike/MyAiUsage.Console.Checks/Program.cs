using System.Diagnostics;
using System.Text.Json;
using MyAiUsage;

var startInfo = CodexAppServer.CreateStartInfo();

Assert(startInfo.FileName == "cmd.exe", "uses the Windows command resolver");
Assert(
    startInfo.ArgumentList.SequenceEqual(["/d", "/c", "codex", "app-server"]),
    "starts codex app-server from PATH"
);
Assert(startInfo.RedirectStandardInput, "redirects stdin");
Assert(startInfo.RedirectStandardOutput, "redirects stdout");
Assert(startInfo.RedirectStandardError, "redirects stderr");
Assert(!startInfo.UseShellExecute, "does not use the shell");
Assert(startInfo.CreateNoWindow, "does not create a window");

using var child = Process.Start(new ProcessStartInfo("powershell")
{
    Arguments = "-NoProfile -Command Start-Sleep -Seconds 30",
    UseShellExecute = false,
}) ?? throw new InvalidOperationException("Could not start check process.");

await CodexAppServer.StopAsync(child);
Assert(child.HasExited, "stops the owned process");

var requestOutput = new StringWriter();
var response = await CodexAppServer.RequestAsync(
    requestOutput,
    new StringReader("{\"method\":\"account/rateLimits/updated\"}\n{\"id\":1,\"result\":{\"ok\":true}}\n"),
    1,
    "initialize",
    new { clientInfo = new { name = "my-ai-usage", version = "0.1.0" } }
);

Assert(
    requestOutput.ToString() == "{\"id\":1,\"method\":\"initialize\",\"params\":{\"clientInfo\":{\"name\":\"my-ai-usage\",\"version\":\"0.1.0\"}}}" + Environment.NewLine,
    "writes one JSON request per line"
);
Assert(response.GetProperty("ok").GetBoolean(), "returns the matching response");

var notificationOutput = new StringWriter();
await CodexAppServer.NotifyAsync(notificationOutput, "initialized");
Assert(
    notificationOutput.ToString() == "{\"method\":\"initialized\"}" + Environment.NewLine,
    "writes one JSON notification per line"
);

await AssertThrowsAsync<InvalidOperationException>(
    () => CodexAppServer.RequestAsync(
        TextWriter.Null,
        new StringReader("{\"id\":2,\"error\":{\"message\":\"denied\"}}\n"),
        2,
        "account/rateLimits/read",
        null
    ),
    "rejects app-server errors"
);

using var multipleBuckets = JsonDocument.Parse("""
{
  "rateLimitsByLimitId": {
    "codex": {
      "limitName": "Codex",
      "primary": { "usedPercent": 17, "windowDurationMins": 300, "resetsAt": 0 },
      "secondary": { "usedPercent": 42, "windowDurationMins": null, "resetsAt": null }
    },
    "other": {
      "limitId": "other",
      "primary": { "usedPercent": 5, "windowDurationMins": 60, "resetsAt": 3600 }
    }
  }
}
""");
Assert(
    RateLimitFormatter.Format(multipleBuckets.RootElement, TimeZoneInfo.Utc).SequenceEqual([
        "Codex — primária: 17% usado | 300 min | reset 1970-01-01 00:00 +00:00",
        "Codex — secundária: 42% usado | duração desconhecida | reset desconhecido",
        "other — primária: 5% usado | 60 min | reset 1970-01-01 01:00 +00:00",
    ]),
    "formats every available rate-limit window"
);

using var fallbackBucket = JsonDocument.Parse("""
{
  "rateLimits": {
    "primary": { "usedPercent": 9 }
  }
}
""");
Assert(
    RateLimitFormatter.Format(fallbackBucket.RootElement, TimeZoneInfo.Utc).SequenceEqual([
        "Codex — primária: 9% usado | duração desconhecida | reset desconhecido",
    ]),
    "falls back to the legacy bucket"
);

Console.WriteLine("Process configuration checks passed.");

static void Assert(bool condition, string message)
{
    if (!condition)
    {
        throw new InvalidOperationException(message);
    }
}

static async Task AssertThrowsAsync<T>(Func<Task> action, string message) where T : Exception
{
    try
    {
        await action();
    }
    catch (T)
    {
        return;
    }

    throw new InvalidOperationException(message);
}
