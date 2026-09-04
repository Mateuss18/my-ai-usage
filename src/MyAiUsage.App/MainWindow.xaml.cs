using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MyAiUsage.Core;

namespace MyAiUsage.App;

public sealed partial class MainWindow : Window
{
    private readonly CodexClient _client = new();
    private readonly SemaphoreSlim _refreshGate = new(1, 1);
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromSeconds(60) };
    private readonly CancellationTokenSource _lifetime = new();
    private RateLimitSnapshot? _lastGoodSnapshot;

    public MainWindow()
    {
        InitializeComponent();
        _timer.Tick += OnTimerTick;
        AppWindow.Changed += OnAppWindowChanged;
        Closed += OnClosed;
    }

    private async void OnLoaded(object sender, RoutedEventArgs e)
    {
        _timer.Start();
        await RefreshAsync(_lifetime.Token);
    }

    private async void OnRefreshClicked(object sender, RoutedEventArgs e) =>
        await RefreshAsync(_lifetime.Token);

    private async void OnTimerTick(object? sender, object e) =>
        await RefreshAsync(_lifetime.Token);

    private async void OnAppWindowChanged(Microsoft.UI.Windowing.AppWindow sender, Microsoft.UI.Windowing.AppWindowChangedEventArgs args)
    {
        if (!args.DidVisibilityChange)
        {
            return;
        }

        if (!sender.IsVisible)
        {
            _timer.Stop();
            return;
        }

        _timer.Start();
        if (_lastGoodSnapshot is null || DateTimeOffset.Now - _lastGoodSnapshot.RetrievedAt > TimeSpan.FromSeconds(60))
        {
            await RefreshAsync(_lifetime.Token);
        }
    }

    private async Task RefreshAsync(CancellationToken cancellationToken)
    {
        try
        {
            await _refreshGate.WaitAsync(cancellationToken);
        }
        catch (OperationCanceledException)
        {
            return;
        }

        try
        {
            SetState("Carregando");
            var snapshot = await _client.ReadRateLimitsAsync(cancellationToken);
            if (snapshot.IsPartial)
            {
                Render(_lastGoodSnapshot ?? snapshot);
                SetState("Dados parciais");
                return;
            }

            _lastGoodSnapshot = snapshot;
            Render(snapshot);
            LastUpdatedText.Text = $"Último snapshot completo: {snapshot.RetrievedAt.ToLocalTime():g}";
            SetState("Disponível");
        }
        catch (CodexClientException error)
        {
            Render(_lastGoodSnapshot);
            var state = error.Kind == CodexClientErrorKind.Cancelled
                ? "Atualização cancelada"
                : MapError(error.Kind);
            SetState(_lastGoodSnapshot is null ? state : $"{state} — Desatualizado");
        }
        finally
        {
            _refreshGate.Release();
        }
    }

    private void Render(RateLimitSnapshot? snapshot)
    {
        QuotaPanel.Children.Clear();
        if (snapshot is null)
        {
            QuotaPanel.Children.Add(new QuotaRing
            {
                BucketName = "Codex",
                Window = new RateLimitWindow("unknown", null, null, null)
            });
            return;
        }

        foreach (var bucket in snapshot.Buckets)
        {
            var bucketPanel = new StackPanel { Spacing = 12 };
            bucketPanel.Children.Add(new TextBlock
            {
                Text = bucket.DisplayName,
                FontSize = 20,
                FontWeight = Microsoft.UI.Text.FontWeights.SemiBold
            });
            var windows = new VariableSizedWrapGrid
            {
                MaximumRowsOrColumns = 3,
                Orientation = Orientation.Horizontal
            };
            foreach (var window in bucket.Windows)
            {
                windows.Children.Add(new QuotaRing { BucketName = bucket.DisplayName, Window = window });
            }
            bucketPanel.Children.Add(windows);
            QuotaPanel.Children.Add(bucketPanel);
        }
    }

    private void SetState(string state) => StatusText.Text = state;

    private static string MapError(CodexClientErrorKind kind) => kind switch
    {
        CodexClientErrorKind.ExecutableNotFound => "Codex ausente",
        CodexClientErrorKind.AuthenticationRequired => "Desconectado",
        CodexClientErrorKind.PartialData => "Dados parciais",
        CodexClientErrorKind.Cancelled => "Atualização cancelada",
        _ => "Falha temporária"
    };

    private async void OnClosed(object sender, WindowEventArgs args)
    {
        _timer.Stop();
        _lifetime.Cancel();
        await _client.DisposeAsync();
        _lifetime.Dispose();
    }
}
