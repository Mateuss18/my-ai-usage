using System.Diagnostics;
using System.Runtime.CompilerServices;
using System.Text.Json;
using MyAiUsage.Core;

CheckParser();
CheckPresentation();
CheckTrayCallback();
await CheckClientAsync();

Console.WriteLine("Core checks passed.");

static void CheckPresentation()
{
    Assert(QuotaPresentation.WindowTitle(300) == "Janela de 5 horas", "formats hours");
    Assert(QuotaPresentation.WindowTitle(10080) == "Janela de 7 dias", "formats days");
    Assert(QuotaPresentation.UsageColor(49) == "green", "uses green below 50");
    Assert(QuotaPresentation.UsageColor(50) == "yellow", "uses yellow at 50");
    Assert(QuotaPresentation.UsageColor(79) == "yellow", "uses yellow below 80");
    Assert(QuotaPresentation.UsageColor(80) == "red", "uses red at 80");
    Assert(QuotaPresentation.UsageColor(null) == "neutral", "uses neutral for unknown");
    Assert(QuotaPresentation.UsageColor(101) == "neutral", "uses neutral outside the valid range");
    Assert(QuotaPresentation.UsageState(42) == "Disponível", "shows available state");
    Assert(QuotaPresentation.UsageState(100) == "Limite atingido", "shows limit-reached state");
    Assert(QuotaPresentation.UsageState(null) == "Uso desconhecido", "shows unknown state");
}

static void CheckParser()
{
    using var document = JsonDocument.Parse("""
    {
      "rateLimitsByLimitId": {
        "codex": {
          "limitName": "Codex",
          "limitId": "codex",
          "primary": { "usedPercent": 17, "windowDurationMins": 300, "resetsAt": 0 },
          "secondary": { "usedPercent": null, "windowDurationMins": 60, "resetsAt": 3600 },
          "tertiary": { "usedPercent": 101, "windowDurationMins": -1, "resetsAt": "invalid" },
          "quaternary": { "usedPercent": "not-a-number", "resetsAt": 7200 }
        }
      }
    }
    """);

    var snapshot = RateLimitParser.Parse(document.RootElement, TimeZoneInfo.Utc);
    Assert(snapshot.Buckets.Count == 1, "reads one bucket");
    Assert(snapshot.Buckets[0].DisplayName == "Codex", "reads bucket display name");
    Assert(snapshot.Buckets[0].Windows.Count == 4, "reads every window object");
    Assert(snapshot.Buckets[0].Windows[0] == new RateLimitWindow(
        "primary", 17, 300, DateTimeOffset.UnixEpoch), "reads a valid window");
    Assert(snapshot.Buckets[0].Windows[1].UsedPercent is null, "null percentage is unknown");
    Assert(snapshot.Buckets[0].Windows[2].UsedPercent is null, "out-of-range percentage is unknown");
    Assert(snapshot.Buckets[0].Windows[2].WindowDurationMins is null, "non-positive duration is unknown");
    Assert(snapshot.Buckets[0].Windows[2].ResetsAt is null, "invalid timestamp is unknown");
    Assert(snapshot.Buckets[0].Windows[3].UsedPercent is null, "non-numeric percentage is unknown");
    Assert(snapshot.Buckets[0].Windows[3].WindowDurationMins is null, "missing duration is unknown");
    Assert(snapshot.IsPartial, "invalid window data is partial");

    using var fallback = JsonDocument.Parse("""
    { "rateLimitsByLimitId": {}, "rateLimits": { "primary": { "usedPercent": 9, "windowDurationMins": 60, "resetsAt": 3600 } } }
    """);
    var fallbackSnapshot = RateLimitParser.Parse(fallback.RootElement, TimeZoneInfo.Utc);
    Assert(fallbackSnapshot.Buckets.Single().Windows.Single().Key == "primary", "uses the fallback bucket");

    using var metadataOnly = JsonDocument.Parse("""
    { "rateLimitsByLimitId": { "codex": { "limitName": "Codex", "limitId": "codex" } } }
    """);
    var partial = AssertThrows<CodexClientException>(
        () => _ = RateLimitParser.Parse(metadataOnly.RootElement, TimeZoneInfo.Utc),
        "rejects a response without windows");
    Assert(partial.Kind == CodexClientErrorKind.PartialData, "classifies missing windows");
    Assert(partial.Message == "Não foi possível ler as janelas de quota do Codex.", "uses the safe partial message");
}

static void CheckTrayCallback()
{
    var tray = (MyAiUsage.App.TrayIcon)RuntimeHelpers.GetUninitializedObject(typeof(MyAiUsage.App.TrayIcon));
    var openCalls = 0;
    typeof(MyAiUsage.App.TrayIcon).GetField("_open", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!
        .SetValue(tray, (Action)(() => openCalls++));

    var callback = new IntPtr(unchecked((long)((0xBEEF << 16) | 0x0202)));
    typeof(MyAiUsage.App.TrayIcon).GetMethod("WndProc", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!.Invoke(
        tray,
        [
            IntPtr.Zero,
            0x8001u,
            UIntPtr.Zero,
            callback
        ]);

    Assert(openCalls == 1, "decodes the LOWORD of a packed tray callback");
    Console.WriteLine("Tray callback check passed.");
}

static async Task CheckClientAsync()
{
    await CheckAuthenticatedSequenceAsync();
    await CheckStartDisposeRaceAsync();
    await CheckAuthenticationErrorAsync();
    await CheckAuthenticationRetryAsync();
    await CheckBrokenPipeAsync();
    await CheckEndOfStreamAsync();
    await CheckInvalidJsonAsync();
    await CheckTimeoutAsync();
    await CheckCancellationAsync();
    await CheckMissingExecutableAsync();
}

static async Task CheckAuthenticatedSequenceAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    setlocal EnableExtensions
    set /a n=0
    for /l %%i in (1,1,4000) do echo stderr-%%i 1>&2
    :loop
    set "line="
    set /p "line="
    if errorlevel 1 exit /b 0
    set /a n+=1 >nul
    if %n%==1 echo(%line%| findstr /c:"initialize" >nul
    if %n%==1 if errorlevel 1 exit /b 97
    if %n%==2 echo(%line%| findstr /c:"initialized" >nul
    if %n%==2 if errorlevel 1 exit /b 97
    if %n%==3 echo(%line%| findstr /c:"account/read" >nul
    if %n%==3 if errorlevel 1 exit /b 97
    if %n%==3 echo(%line%| findstr /c:"refreshToken" >nul
    if %n%==3 if errorlevel 1 exit /b 97
    if %n%==4 echo(%line%| findstr /c:"account/rateLimits/read" >nul
    if %n%==4 if errorlevel 1 exit /b 97
    if %n%==5 echo(%line%| findstr /c:"account/read" >nul
    if %n%==5 if errorlevel 1 exit /b 97
    if %n%==6 echo(%line%| findstr /c:"account/rateLimits/read" >nul
    if %n%==6 if errorlevel 1 exit /b 97
    if %n%==1 echo {"id":1,"result":{"ok":true}}
    if %n%==2 echo {"method":"account/rateLimits/updated"}
    if %n%==3 echo {"id":2,"result":{"account":{"type":"chatgpt"}}}
    if %n%==4 echo {"id":3,"result":{"rateLimits":{"primary":{"usedPercent":42,"windowDurationMins":300,"resetsAt":0}}}}
    if %n%==5 echo {"id":4,"result":{"account":{"type":"chatgpt"}}}
    if %n%==6 echo {"id":5,"result":{"rateLimits":{"primary":{"usedPercent":42,"windowDurationMins":300,"resetsAt":0}}}}
    goto loop
    """, async client =>
    {
        await client.StartAsync();
        var snapshots = await Task.WhenAll(client.ReadRateLimitsAsync(), client.ReadRateLimitsAsync());
        Assert(snapshots.All(snapshot => snapshot.Buckets.Single().Windows.Single().UsedPercent == 42), "serializes rate-limit reads");
    });
}

static async Task CheckAuthenticationErrorAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    setlocal
    echo {"id":1,"error":{"message":"authentication required"}}
    """, async client =>
    {
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(), "classifies authentication errors");
        Assert(error.Kind == CodexClientErrorKind.AuthenticationRequired, "uses the authentication error kind");
    });
}

static async Task CheckAuthenticationRetryAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    echo {"id":1,"error":{"message":"authentication required"}}
    """, async client =>
    {
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(), "classifies authentication errors before retry");
        Assert(error.Kind == CodexClientErrorKind.AuthenticationRequired, "keeps the authentication error kind before retry");

        var fakeCodex = Path.Combine(Environment.GetEnvironmentVariable("PATH")!.Split(';')[0], "codex.cmd");
        await File.WriteAllTextAsync(fakeCodex, """
        @echo off
        setlocal EnableExtensions
        :loop
        set "line="
        set /p "line="
        if errorlevel 1 exit /b 0
        echo(%line%| findstr /c:"initialize" >nul
        if not errorlevel 1 echo {"id":2,"result":{"ok":true}}
        goto loop
        """.Replace("\n", "\r\n"));

        await client.StartAsync();
    });
}

static async Task CheckBrokenPipeAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    setlocal
    echo {"id":1,"result":{"ok":true}}
    :loop
    set "line="
    set /p "line="
    if errorlevel 1 exit /b 0
    goto loop
    """, async client =>
    {
        await client.StartAsync();
        SetClientInput(client, new StreamWriter(new FailingWriteStream()));

        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.ReadRateLimitsAsync(), "classifies broken pipes");
        Assert(error.Kind == CodexClientErrorKind.EndOfStream, "uses the EOF error kind for broken pipes");
    });
}

static async Task CheckEndOfStreamAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    exit /b 0
    """, async client =>
    {
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(), "classifies EOF");
        Assert(error.Kind == CodexClientErrorKind.EndOfStream, "uses the EOF error kind");
    });
}

static async Task CheckStartDisposeRaceAsync()
{
    for (var attempt = 0; attempt < 8; attempt++)
    {
        await WithFakeCodexAsync("""
        @echo off
        :loop
        set "line="
        set /p "line="
        if errorlevel 1 exit /b 0
        goto loop
        """, async client =>
        {
            var startTask = Task.Run(() => client.StartAsync());
            var disposeTask = Task.Run(() => client.DisposeAsync().AsTask());
            await disposeTask;

            try
            {
                await startTask;
            }
            catch (CodexClientException)
            {
            }
            catch (ObjectDisposedException)
            {
            }

            var process = GetClientProcess(client);
            if (process is null)
            {
                return;
            }

            try
            {
                Assert(await WaitForExitAsync(process), "disposes a process published during a start race");
            }
            finally
            {
                await StopProcessAsync(process);
            }
        });
    }
}

static async Task CheckInvalidJsonAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    echo {invalid
    """, async client =>
    {
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(), "classifies invalid JSON");
        Assert(error.Kind == CodexClientErrorKind.InvalidJson, "uses the invalid JSON error kind");
    });
}

static async Task CheckTimeoutAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    ping -n 12 127.0.0.1 >nul
    """, async client =>
    {
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(), "classifies timeout");
        Assert(error.Kind == CodexClientErrorKind.Timeout, "uses the timeout error kind: " + error.Kind);
    });
}

static async Task CheckCancellationAsync()
{
    await WithFakeCodexAsync("""
    @echo off
    ping -n 30 127.0.0.1 >nul
    """, async client =>
    {
        using var cancellation = new CancellationTokenSource(TimeSpan.FromMilliseconds(100));
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(cancellation.Token), "classifies cancellation");
        Assert(error.Kind == CodexClientErrorKind.Cancelled, "uses the cancellation error kind");
    });
}

static async Task CheckMissingExecutableAsync()
{
    var previousPath = Environment.GetEnvironmentVariable("PATH");
    var systemRoot = Environment.GetEnvironmentVariable("SystemRoot") ?? "C:\\Windows";
    Environment.SetEnvironmentVariable("PATH", Path.Combine(systemRoot, "System32") + ";" + systemRoot);

    try
    {
        await using var client = new CodexClient();
        var error = await AssertThrowsAsync<CodexClientException>(
            () => client.StartAsync(), "classifies a missing executable");
        Assert(error.Kind == CodexClientErrorKind.ExecutableNotFound, "uses the executable-not-found error kind: " + error.Kind);
    }
    finally
    {
        Environment.SetEnvironmentVariable("PATH", previousPath);
    }
}

static async Task WithFakeCodexAsync(string script, Func<CodexClient, Task> action)
{
    var directory = Path.Combine(Path.GetTempPath(), "my-ai-usage-checks-" + Guid.NewGuid().ToString("N"));
    Directory.CreateDirectory(directory);
    var previousPath = Environment.GetEnvironmentVariable("PATH");
    var systemRoot = Environment.GetEnvironmentVariable("SystemRoot") ?? "C:\\Windows";
    Environment.SetEnvironmentVariable("PATH", directory + ";" + Path.Combine(systemRoot, "System32") + ";" + systemRoot);
    await File.WriteAllTextAsync(Path.Combine(directory, "codex.cmd"), script.Replace("\n", "\r\n"));

    try
    {
        await using var client = new CodexClient();
        await action(client);
        await client.DisposeAsync();
        await client.DisposeAsync();
    }
    finally
    {
        Environment.SetEnvironmentVariable("PATH", previousPath);
        Directory.Delete(directory, recursive: true);
    }
}

static T AssertThrows<T>(Action action, string message) where T : Exception
{
    try
    {
        action();
    }
    catch (T exception)
    {
        return exception;
    }

    throw new InvalidOperationException(message);
}

static async Task<T> AssertThrowsAsync<T>(Func<Task> action, string message) where T : Exception
{
    try
    {
        await action();
    }
    catch (T exception)
    {
        return exception;
    }

    throw new InvalidOperationException(message);
}

static void Assert(bool condition, string message)
{
    if (!condition)
    {
        throw new InvalidOperationException(message);
    }
}

static Process? GetClientProcess(CodexClient client) =>
    typeof(CodexClient)
        .GetField("_process", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)
        ?.GetValue(client) as Process;

static void SetClientInput(CodexClient client, StreamWriter input) =>
    typeof(CodexClient)
        .GetField("_input", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!
        .SetValue(client, input);

static async Task<bool> WaitForExitAsync(Process process)
{
    for (var attempt = 0; attempt < 40; attempt++)
    {
        try
        {
            if (process.HasExited)
            {
                return true;
            }
        }
        catch (InvalidOperationException)
        {
            return true;
        }

        await Task.Delay(50);
    }

    try
    {
        return process.HasExited;
    }
    catch (InvalidOperationException)
    {
        return true;
    }
}

static async Task StopProcessAsync(Process process)
{
    try
    {
        if (!process.HasExited)
        {
            process.Kill(entireProcessTree: true);
        }

        await process.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
    }
    catch (ArgumentException)
    {
    }
    catch (InvalidOperationException)
    {
    }
    catch (System.ComponentModel.Win32Exception)
    {
    }
    catch (TimeoutException)
    {
    }
}

sealed class FailingWriteStream : MemoryStream
{
    public override ValueTask WriteAsync(ReadOnlyMemory<byte> buffer, CancellationToken cancellationToken = default) =>
        ValueTask.FromException(new IOException("broken pipe"));
}
