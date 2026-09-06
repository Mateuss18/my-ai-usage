using Microsoft.UI.Xaml;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.Windows.AppLifecycle;
using WinRT.Interop;

namespace MyAiUsage.App;

public partial class App : Application
{
    private const string InstanceKey = "my-ai-usage";
    private readonly object _exitLock = new();
    private AppInstance? _instance;
    private MainWindow? _window;
    private DispatcherQueue? _dispatcherQueue;
    private TrayIcon? _tray;
    private Task? _exitTask;
    private bool _isExiting;

    internal bool IsExiting => _isExiting;

    public App()
    {
        InitializeComponent();
    }

    protected override async void OnLaunched(LaunchActivatedEventArgs args)
    {
        var activation = AppInstance.GetCurrent().GetActivatedEventArgs();
        var instance = AppInstance.FindOrRegisterForKey(InstanceKey);
        if (!instance.IsCurrent)
        {
            await instance.RedirectActivationToAsync(activation);
            Current.Exit();
            return;
        }

        _instance = instance;
        _window ??= new MainWindow();
        _dispatcherQueue = _window.DispatcherQueue;
        _instance.Activated += OnActivated;
        if (activation.Kind == ExtendedActivationKind.StartupTask)
        {
            _window.AppWindow.Hide();
        }
        else
        {
            OpenOrRestoreWindow();
        }

        _tray ??= new TrayIcon(WindowNative.GetWindowHandle(_window), OpenOrRestoreWindow, ExitApplicationAsync);
    }

    public void OpenOrRestoreWindow()
    {
        if (_window is null || _isExiting)
        {
            return;
        }

        _window.AppWindow.Show();
        if (_window.AppWindow.Presenter is OverlappedPresenter { State: OverlappedPresenterState.Minimized } presenter)
        {
            presenter.Restore();
        }
        _window.Activate();
    }

    public void HideWindow() => _window?.AppWindow.Hide();

    public Task ExitApplicationAsync()
    {
        lock (_exitLock)
        {
            return _exitTask ??= ExitCoreAsync();
        }
    }

    private void OnActivated(object? sender, AppActivationArguments args) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            if (args.Kind != ExtendedActivationKind.StartupTask)
            {
                OpenOrRestoreWindow();
            }
        });

    private async Task ExitCoreAsync()
    {
        _isExiting = true;
        try
        {
            if (_window is not null)
            {
                await _window.DisposeAsync();
            }
        }
        finally
        {
            _tray?.Dispose();
            _tray = null;
            Current.Exit();
        }
    }
}
