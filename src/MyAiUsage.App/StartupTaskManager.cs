using NativeStartupTaskState = Windows.ApplicationModel.StartupTaskState;
using Windows.ApplicationModel;

namespace MyAiUsage.App;

public enum StartupTaskState
{
    Enabled,
    EnabledByPolicy,
    Disabled,
    DisabledByUser,
    DisabledByPolicy,
    Unavailable
}

public sealed class StartupTaskManager
{
    public const string TaskId = "MyAiUsageStartup";

    private StartupTask? _task;

    public StartupTaskState State { get; private set; } = StartupTaskState.Unavailable;

    public string Reason { get; private set; } =
        "O auto-start só está disponível quando o app está instalado como pacote MSIX.";

    public bool CanChange => State is StartupTaskState.Enabled or StartupTaskState.Disabled;

    public async Task<StartupTaskState> GetStateAsync()
    {
        try
        {
            _task = await StartupTask.GetAsync(TaskId);
            return ApplyState(_task.State);
        }
        catch (Exception)
        {
            _task = null;
            return SetUnavailable(
                "O auto-start está indisponível fora do pacote MSIX ou devido a uma falha do Windows.");
        }
    }

    public async Task<StartupTaskState> SetEnabledAsync(bool enabled)
    {
        if (_task is null)
        {
            await GetStateAsync();
        }

        if (_task is null)
        {
            return State;
        }

        try
        {
            if (enabled && State == StartupTaskState.Disabled)
            {
                return ApplyState(await _task.RequestEnableAsync());
            }

            if (!enabled && State == StartupTaskState.Enabled)
            {
                _task.Disable();
                return await GetStateAsync();
            }

            return State;
        }
        catch (Exception)
        {
            return SetUnavailable("O Windows não conseguiu atualizar o auto-start.");
        }
    }

    private StartupTaskState ApplyState(NativeStartupTaskState state)
    {
        State = state switch
        {
            NativeStartupTaskState.Enabled => StartupTaskState.Enabled,
            NativeStartupTaskState.EnabledByPolicy => StartupTaskState.EnabledByPolicy,
            NativeStartupTaskState.Disabled => StartupTaskState.Disabled,
            NativeStartupTaskState.DisabledByUser => StartupTaskState.DisabledByUser,
            NativeStartupTaskState.DisabledByPolicy => StartupTaskState.DisabledByPolicy,
            _ => StartupTaskState.Unavailable
        };

        Reason = State switch
        {
            StartupTaskState.Enabled =>
                "O app será iniciado automaticamente com o Windows.",
            StartupTaskState.EnabledByPolicy =>
                "O app será iniciado automaticamente; o administrador do Windows controla esta configuração.",
            StartupTaskState.Disabled =>
                "Ative o controle para iniciar o app com o Windows.",
            StartupTaskState.DisabledByUser =>
                "Reative esta tarefa nas configurações de inicialização do Windows.",
            StartupTaskState.DisabledByPolicy =>
                "O administrador do Windows controla esta configuração.",
            _ =>
                "O auto-start está indisponível fora do pacote MSIX ou devido a uma falha do Windows."
        };

        return State;
    }

    private StartupTaskState SetUnavailable(string reason)
    {
        State = StartupTaskState.Unavailable;
        Reason = reason;
        return State;
    }
}
