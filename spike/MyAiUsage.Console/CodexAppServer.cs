using System.Diagnostics;
using System.Text.Json;

namespace MyAiUsage;

public static class CodexAppServer
{
    public static ProcessStartInfo CreateStartInfo()
    {
        var startInfo = new ProcessStartInfo("cmd.exe")
        {
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
        };
        startInfo.ArgumentList.Add("/d");
        startInfo.ArgumentList.Add("/c");
        startInfo.ArgumentList.Add("codex");
        startInfo.ArgumentList.Add("app-server");
        return startInfo;
    }

    public static Process Start() =>
        Process.Start(CreateStartInfo())
        ?? throw new InvalidOperationException("Não foi possível iniciar o codex app-server.");

    public static async Task StopAsync(Process process)
    {
        if (process.HasExited)
        {
            return;
        }

        process.Kill(entireProcessTree: true);
        await process.WaitForExitAsync();
    }

    public static async Task<JsonElement> RequestAsync(
        TextWriter input,
        TextReader output,
        int id,
        string method,
        object? parameters,
        CancellationToken cancellationToken = default
    )
    {
        var message = JsonSerializer.Serialize(new { id, method, @params = parameters });
        await input.WriteLineAsync(message.AsMemory(), cancellationToken);
        await input.FlushAsync(cancellationToken);

        while (true)
        {
            var line = await output.ReadLineAsync(cancellationToken)
                ?? throw new EndOfStreamException("O codex app-server encerrou sem responder.");
            using var response = JsonDocument.Parse(line);
            var root = response.RootElement;

            if (!root.TryGetProperty("id", out var responseId)
                || !responseId.TryGetInt32(out var value)
                || value != id)
            {
                continue;
            }

            if (root.TryGetProperty("error", out var error))
            {
                var serverMessage = error.ValueKind == JsonValueKind.Object
                    && error.TryGetProperty("message", out var errorProperty)
                    && errorProperty.ValueKind == JsonValueKind.String
                        ? errorProperty.GetString()
                        : null;

                if (serverMessage?.Contains("authentication", StringComparison.OrdinalIgnoreCase) == true)
                {
                    throw new InvalidOperationException("É necessário autenticar o Codex para consultar as quotas.");
                }

                throw new InvalidOperationException("O codex app-server retornou um erro.");
            }

            if (!root.TryGetProperty("result", out var result))
            {
                throw new InvalidDataException("Resposta do codex app-server sem resultado.");
            }

            return result.Clone();
        }
    }

    public static async Task NotifyAsync(
        TextWriter input,
        string method,
        CancellationToken cancellationToken = default
    )
    {
        var message = JsonSerializer.Serialize(new { method });
        await input.WriteLineAsync(message.AsMemory(), cancellationToken);
        await input.FlushAsync(cancellationToken);
    }
}
