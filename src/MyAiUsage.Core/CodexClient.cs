using System.ComponentModel;
using System.Diagnostics;
using System.Text.Json;

namespace MyAiUsage.Core;

public sealed class CodexClient : IAsyncDisposable
{
    private static readonly TimeSpan RequestTimeout = TimeSpan.FromSeconds(10);
    private readonly SemaphoreSlim _startGate = new(1, 1);
    private readonly SemaphoreSlim _readGate = new(1, 1);
    private readonly CancellationTokenSource _lifetime = new();
    private readonly object _disposeLock = new();
    private Process? _process;
    private StreamWriter? _input;
    private StreamReader? _output;
    private Task? _stderrDrain;
    private Task? _disposeTask;
    private int _nextRequestId;

    public async Task StartAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        try
        {
            await _startGate.WaitAsync(cancellationToken);
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw Cancelled();
        }

        try
        {
            lock (_disposeLock)
            {
                ThrowIfDisposed();

                if (_process is not null)
                {
                    if (!_process.HasExited)
                    {
                        return;
                    }

                    throw EndOfStream();
                }

                var startInfo = new ProcessStartInfo("cmd.exe")
                {
                    UseShellExecute = false,
                    CreateNoWindow = true,
                    RedirectStandardInput = true,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true
                };
                startInfo.ArgumentList.Add("/d");
                startInfo.ArgumentList.Add("/c");
                startInfo.ArgumentList.Add("codex");
                startInfo.ArgumentList.Add("app-server");

                Process process;
                try
                {
                    process = Process.Start(startInfo)
                        ?? throw new CodexClientException(
                            CodexClientErrorKind.ExecutableNotFound,
                            "Não foi possível iniciar o Codex.");
                }
                catch (Win32Exception)
                {
                    throw new CodexClientException(
                        CodexClientErrorKind.ExecutableNotFound,
                        "Não foi possível iniciar o Codex.");
                }

                _process = process;
                _input = process.StandardInput;
                _output = process.StandardOutput;
                _stderrDrain = DrainStandardErrorAsync(process.StandardError, _lifetime.Token);
            }

            using var timeout = new CancellationTokenSource(RequestTimeout);
            using var operation = CancellationTokenSource.CreateLinkedTokenSource(
                cancellationToken,
                timeout.Token,
                _lifetime.Token);
            _ = await RequestAsync(
                "initialize",
                new { clientInfo = new { name = "my-ai-usage", version = "0.1.0" } },
                operation.Token);
            await NotifyAsync("initialized", operation.Token);
        }
        catch (ObjectDisposedException)
        {
            throw;
        }
        catch (CodexClientException)
        {
            await ResetTransportAsync();
            throw;
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            await ResetTransportAsync();
            throw Cancelled();
        }
        catch (OperationCanceledException)
        {
            await ResetTransportAsync();
            throw new CodexClientException(
                CodexClientErrorKind.Timeout,
                "A conexão com o Codex excedeu o tempo limite.");
        }
        catch (JsonException)
        {
            await ResetTransportAsync();
            throw new CodexClientException(
                CodexClientErrorKind.InvalidJson,
                "O Codex retornou JSON inválido.");
        }
        catch (IOException error)
        {
            await ResetTransportAsync();
            throw new CodexClientException(
                CodexClientErrorKind.EndOfStream,
                "O codex app-server encerrou sem responder.",
                error);
        }
        catch (Exception)
        {
            await ResetTransportAsync();
            throw new CodexClientException(
                CodexClientErrorKind.ProtocolError,
                "O Codex não aceitou a solicitação.");
        }
        finally
        {
            _startGate.Release();
        }
    }

    public async Task<RateLimitSnapshot> ReadRateLimitsAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();
        await StartAsync(cancellationToken);

        try
        {
            await _readGate.WaitAsync(cancellationToken);
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw Cancelled();
        }

        try
        {
            using var timeout = new CancellationTokenSource(RequestTimeout);
            using var operation = CancellationTokenSource.CreateLinkedTokenSource(
                cancellationToken,
                timeout.Token,
                _lifetime.Token);
            _ = await RequestAsync("account/read", new { refreshToken = false }, operation.Token);
            var result = await RequestAsync("account/rateLimits/read", null, operation.Token);
            return RateLimitParser.Parse(result, TimeZoneInfo.Local);
        }
        catch (CodexClientException error) when (
            error.Kind is CodexClientErrorKind.EndOfStream
                or CodexClientErrorKind.InvalidJson
                or CodexClientErrorKind.ProtocolError)
        {
            await ResetTransportAsync();
            throw;
        }
        catch (CodexClientException)
        {
            throw;
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            await ResetTransportAsync();
            throw Cancelled();
        }
        catch (OperationCanceledException)
        {
            await ResetTransportAsync();
            throw new CodexClientException(
                CodexClientErrorKind.Timeout,
                "A conexão com o Codex excedeu o tempo limite.");
        }
        catch (JsonException)
        {
            throw new CodexClientException(
                CodexClientErrorKind.InvalidJson,
                "O Codex retornou JSON inválido.");
        }
        catch (IOException error)
        {
            await ResetTransportAsync();
            throw new CodexClientException(
                CodexClientErrorKind.EndOfStream,
                "O codex app-server encerrou sem responder.",
                error);
        }
        finally
        {
            _readGate.Release();
        }
    }

    public ValueTask DisposeAsync()
    {
        lock (_disposeLock)
        {
            return new ValueTask(_disposeTask ??= DisposeCoreAsync());
        }
    }

    private async Task<JsonElement> RequestAsync(string method, object? parameters, CancellationToken cancellationToken)
    {
        var input = _input ?? throw new CodexClientException(
            CodexClientErrorKind.ProtocolError,
            "O cliente do Codex não foi iniciado.");
        var output = _output ?? throw new CodexClientException(
            CodexClientErrorKind.ProtocolError,
            "O cliente do Codex não foi iniciado.");
        var id = Interlocked.Increment(ref _nextRequestId);
        var message = JsonSerializer.Serialize(new { id, method, @params = parameters });
        await input.WriteLineAsync(message.AsMemory(), cancellationToken);
        await input.FlushAsync(cancellationToken);

        while (true)
        {
            var line = await ReadLineAsync(output, cancellationToken);
            if (line is null)
            {
                throw EndOfStream();
            }

            JsonDocument response;
            try
            {
                response = JsonDocument.Parse(line);
            }
            catch (JsonException)
            {
                throw new CodexClientException(
                    CodexClientErrorKind.InvalidJson,
                    "O Codex retornou JSON inválido.");
            }

            using (response)
            {
                var root = response.RootElement;
                if (root.ValueKind != JsonValueKind.Object)
                {
                    throw new CodexClientException(
                        CodexClientErrorKind.ProtocolError,
                        "Resposta do Codex app-server inválida.");
                }

                if (!root.TryGetProperty("id", out var responseId)
                    || !responseId.TryGetInt32(out var value)
                    || value != id)
                {
                    continue;
                }

                if (root.TryGetProperty("error", out var error))
                {
                    if (IsAuthenticationError(error))
                    {
                        throw new CodexClientException(
                            CodexClientErrorKind.AuthenticationRequired,
                            "É necessário autenticar o Codex para consultar as quotas.");
                    }

                    throw new CodexClientException(
                        CodexClientErrorKind.ProtocolError,
                        "O Codex app-server retornou um erro.");
                }

                if (!root.TryGetProperty("result", out var result))
                {
                    throw new CodexClientException(
                        CodexClientErrorKind.ProtocolError,
                        "Resposta do Codex app-server sem resultado.");
                }

                return result.Clone();
            }
        }
    }

    private async Task NotifyAsync(string method, CancellationToken cancellationToken)
    {
        var input = _input ?? throw new CodexClientException(
            CodexClientErrorKind.ProtocolError,
            "O cliente do Codex não foi iniciado.");
        var message = JsonSerializer.Serialize(new { method });
        await input.WriteLineAsync(message.AsMemory(), cancellationToken);
        await input.FlushAsync(cancellationToken);
    }

    private async Task DisposeCoreAsync()
    {
        _lifetime.Cancel();
        await ResetTransportAsync();
        _lifetime.Dispose();
    }

    private async Task ResetTransportAsync()
    {
        Process? process;
        StreamWriter? input;
        StreamReader? output;
        Task? stderrDrain;

        lock (_disposeLock)
        {
            process = _process;
            input = _input;
            output = _output;
            stderrDrain = _stderrDrain;
            _process = null;
            _input = null;
            _output = null;
            _stderrDrain = null;
        }

        try
        {
            input?.Close();
        }
        catch (IOException)
        {
        }

        if (process is not null)
        {
            try
            {
                if (!process.HasExited)
                {
                    process.Kill(entireProcessTree: true);
                }
            }
            catch (InvalidOperationException)
            {
            }
            catch (Win32Exception)
            {
            }

            try
            {
                await process.WaitForExitAsync();
            }
            catch (InvalidOperationException)
            {
            }
        }

        if (stderrDrain is not null)
        {
            try
            {
                await stderrDrain;
            }
            catch (OperationCanceledException)
            {
            }
        }

        output?.Dispose();
        input?.Dispose();
        process?.Dispose();
    }

    private CodexClientException EndOfStream()
    {
        try
        {
            if (_process?.HasExited == false)
            {
                _process.WaitForExit(250);
            }

            if (_process?.HasExited == true && _process.ExitCode is 9009 or 1)
            {
                return new CodexClientException(
                    CodexClientErrorKind.ExecutableNotFound,
                    "Não foi possível encontrar o Codex no PATH.");
            }
        }
        catch (InvalidOperationException)
        {
        }

        return new CodexClientException(
            CodexClientErrorKind.EndOfStream,
            "O codex app-server encerrou sem responder.");
    }

    private static bool IsAuthenticationError(JsonElement error)
    {
        if (error.ValueKind != JsonValueKind.Object
            || !error.TryGetProperty("message", out var message)
            || message.ValueKind != JsonValueKind.String)
        {
            return false;
        }

        var value = message.GetString();
        return value?.Contains("authentication", StringComparison.OrdinalIgnoreCase) == true
            || value?.Contains("unauthorized", StringComparison.OrdinalIgnoreCase) == true
            || value?.Contains("not authenticated", StringComparison.OrdinalIgnoreCase) == true;
    }

    private static async Task DrainStandardErrorAsync(StreamReader error, CancellationToken cancellationToken)
    {
        var buffer = new char[4096];
        try
        {
            while (await error.ReadAsync(buffer.AsMemory(), cancellationToken) > 0)
            {
            }
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
        }
    }

    private static async Task<string?> ReadLineAsync(StreamReader output, CancellationToken cancellationToken)
    {
        var read = output.ReadLineAsync();
        var cancellation = Task.Delay(Timeout.InfiniteTimeSpan, cancellationToken);
        var completed = await Task.WhenAny(read, cancellation);
        if (completed == cancellation)
        {
            cancellationToken.ThrowIfCancellationRequested();
        }

        return await read;
    }

    private void ThrowIfDisposed()
    {
        if (_disposeTask is not null)
        {
            throw new ObjectDisposedException(nameof(CodexClient));
        }
    }

    private static CodexClientException Cancelled() => new(
        CodexClientErrorKind.Cancelled,
        "A atualização do Codex foi cancelada.");
}
