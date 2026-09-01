using MyAiUsage;

using var appServer = CodexAppServer.Start();
using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(10));

try
{
    _ = await CodexAppServer.RequestAsync(
        appServer.StandardInput,
        appServer.StandardOutput,
        1,
        "initialize",
        new { clientInfo = new { name = "my-ai-usage", version = "0.1.0" } },
        timeout.Token
    );
    Console.WriteLine("initialize OK.");

    await CodexAppServer.NotifyAsync(appServer.StandardInput, "initialized", timeout.Token);

    var rateLimits = await CodexAppServer.RequestAsync(
        appServer.StandardInput,
        appServer.StandardOutput,
        2,
        "account/rateLimits/read",
        null,
        timeout.Token
    );

    foreach (var line in RateLimitFormatter.Format(rateLimits, TimeZoneInfo.Local))
    {
        Console.WriteLine(line);
    }
}
finally
{
    await CodexAppServer.StopAsync(appServer);
}
