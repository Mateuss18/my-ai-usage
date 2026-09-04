using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using MyAiUsage.Core;

namespace MyAiUsage.App;

public sealed partial class QuotaRing : UserControl
{
    private const double DashLength = 42.1;
    private RateLimitWindow _window = new("unknown", null, null, null);
    private string _bucketName = "Codex";

    public QuotaRing()
    {
        InitializeComponent();
        Update();
    }

    public RateLimitWindow Window
    {
        get => _window;
        set { _window = value; Update(); }
    }

    public string BucketName
    {
        get => _bucketName;
        set { _bucketName = value; Update(); }
    }

    public string AccessibleDescription
    {
        get => AutomationProperties.GetName(this);
        set => AutomationProperties.SetName(this, value);
    }

    private void Update()
    {
        var percent = _window.UsedPercent is >= 0 and <= 100 ? _window.UsedPercent : null;
        var title = QuotaPresentation.WindowTitle(_window.WindowDurationMins);
        var usage = percent is int value ? $"{value}% usado" : "Uso desconhecido";
        var reset = _window.ResetsAt is DateTimeOffset resetsAt
            ? $"Reset: {resetsAt.ToLocalTime():g}"
            : "Reset desconhecido";
        var state = QuotaPresentation.UsageState(percent);

        TitleText.Text = title;
        PercentText.Text = percent is int number ? $"{number}%" : "?";
        ResetText.Text = reset;
        StateText.Text = state;
        Progress.Stroke = new SolidColorBrush(ToColor(QuotaPresentation.UsageColor(percent)));
        Progress.StrokeDashArray = percent is int used
            ? new DoubleCollection { DashLength * used / 100, DashLength }
            : new DoubleCollection { 0, DashLength };
        AccessibleDescription = $"{_bucketName}, {title}, {usage}, {reset}, {state}";
    }

    private static Windows.UI.Color ToColor(string usageColor) => usageColor switch
    {
        "green" => ColorHelper.FromArgb(255, 16, 124, 65),
        "yellow" => ColorHelper.FromArgb(255, 184, 134, 11),
        "red" => ColorHelper.FromArgb(255, 196, 43, 28),
        _ => ColorHelper.FromArgb(255, 102, 112, 122)
    };
}
