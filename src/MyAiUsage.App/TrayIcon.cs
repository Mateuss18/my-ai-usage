using System.ComponentModel;
using System.Runtime.InteropServices;

namespace MyAiUsage.App;

internal sealed class TrayIcon : IDisposable
{
    private const int GwlWndProc = -4;
    private const uint WmApp = 0x8000;
    private const uint CallbackMessage = WmApp + 1;
    private const uint WmLButtonUp = 0x0202;
    private const uint WmRButtonUp = 0x0205;
    private const uint WmNull = 0x0000;
    private const uint NifMessage = 0x00000001;
    private const uint NifIcon = 0x00000002;
    private const uint NifTip = 0x00000004;
    private const uint NimAdd = 0x00000000;
    private const uint NimDelete = 0x00000002;
    private const uint NimSetVersion = 0x00000004;
    private const uint NotifyIconVersion4 = 4;
    private const uint MfString = 0x00000000;
    private const uint TpmRightButton = 0x0002;
    private const uint TpmRetCmd = 0x0100;
    private const uint OpenCommand = 1;
    private const uint ExitCommand = 2;
    private const uint IdiApplication = 32512;

    private readonly IntPtr _windowHandle;
    private readonly Action _open;
    private readonly Func<Task> _exit;
    private readonly WindowProcDelegate _windowProc;
    private readonly uint _taskbarCreatedMessage;
    private IntPtr _previousWindowProc;
    private IntPtr _icon;
    private int _iconAdded;
    private int _disposed;

    public TrayIcon(IntPtr windowHandle, Action open, Func<Task> exit)
    {
        ArgumentNullException.ThrowIfNull(open);
        ArgumentNullException.ThrowIfNull(exit);

        if (windowHandle == IntPtr.Zero)
        {
            throw new ArgumentException("A tray icon requires a window handle.", nameof(windowHandle));
        }

        _windowHandle = windowHandle;
        _open = open;
        _exit = exit;
        _windowProc = WndProc;
        _taskbarCreatedMessage = RegisterWindowMessage("TaskbarCreated");
        if (_taskbarCreatedMessage == 0)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }

        _icon = CopyIcon(LoadIcon(IntPtr.Zero, IdiApplication));
        if (_icon == IntPtr.Zero)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }

        try
        {
            _previousWindowProc = SetWindowLongPtr(_windowHandle, GwlWndProc, Marshal.GetFunctionPointerForDelegate(_windowProc));
            if (_previousWindowProc == IntPtr.Zero && Marshal.GetLastWin32Error() != 0)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error());
            }

            AddIcon();
        }
        catch
        {
            Dispose();
            throw;
        }
    }

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0)
        {
            return;
        }

        if (Interlocked.Exchange(ref _iconAdded, 0) != 0)
        {
            var data = CreateNotifyIconData();
            ShellNotifyIcon(NimDelete, ref data);
        }

        if (_previousWindowProc != IntPtr.Zero)
        {
            SetWindowLongPtr(_windowHandle, GwlWndProc, _previousWindowProc);
            _previousWindowProc = IntPtr.Zero;
        }

        if (_icon != IntPtr.Zero)
        {
            DestroyIcon(_icon);
            _icon = IntPtr.Zero;
        }
    }

    private void AddIcon()
    {
        if (Volatile.Read(ref _disposed) != 0)
        {
            return;
        }

        var data = CreateNotifyIconData();
        if (!ShellNotifyIcon(NimAdd, ref data))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }

        Interlocked.Exchange(ref _iconAdded, 1);
        data.uTimeoutOrVersion = NotifyIconVersion4;
        ShellNotifyIcon(NimSetVersion, ref data);
    }

    private NOTIFYICONDATAW CreateNotifyIconData() => new()
    {
        cbSize = (uint)Marshal.SizeOf<NOTIFYICONDATAW>(),
        hWnd = _windowHandle,
        uID = 1,
        uFlags = NifMessage | NifIcon | NifTip,
        uCallbackMessage = CallbackMessage,
        hIcon = _icon,
        szTip = "My AI Usage",
        szInfo = string.Empty,
        szInfoTitle = string.Empty
    };

    private IntPtr WndProc(IntPtr windowHandle, uint message, UIntPtr wParam, IntPtr lParam)
    {
        if (message == CallbackMessage)
        {
            switch (unchecked((uint)lParam.ToInt64()) & 0xFFFF)
            {
                case WmLButtonUp:
                    _open();
                    return IntPtr.Zero;
                case WmRButtonUp:
                    ShowMenu();
                    return IntPtr.Zero;
            }
        }
        else if (message == _taskbarCreatedMessage)
        {
            try
            {
                AddIcon();
            }
            catch (Win32Exception)
            {
                // Explorer can still be restarting; its next TaskbarCreated is another chance.
            }

            return IntPtr.Zero;
        }

        return CallWindowProc(_previousWindowProc, windowHandle, message, wParam, lParam);
    }

    private void ShowMenu()
    {
        if (Volatile.Read(ref _disposed) != 0)
        {
            return;
        }

        var menu = CreatePopupMenu();
        if (menu == IntPtr.Zero)
        {
            return;
        }

        try
        {
            AppendMenu(menu, MfString, OpenCommand, "Abrir");
            AppendMenu(menu, MfString, ExitCommand, "Sair");
            SetForegroundWindow(_windowHandle);
            if (!GetCursorPos(out var point))
            {
                return;
            }

            var command = TrackPopupMenuEx(menu, TpmRightButton | TpmRetCmd, point.X, point.Y, _windowHandle, IntPtr.Zero);
            if (command == OpenCommand)
            {
                _open();
            }
            else if (command == ExitCommand)
            {
                _ = ExitAsync();
            }

            PostMessage(_windowHandle, WmNull, UIntPtr.Zero, IntPtr.Zero);
        }
        finally
        {
            DestroyMenu(menu);
        }
    }

    private async Task ExitAsync()
    {
        try
        {
            await _exit();
        }
        catch
        {
            // The app owns the shutdown task and remains responsible for reporting its errors.
        }
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct NOTIFYICONDATAW
    {
        public uint cbSize;
        public IntPtr hWnd;
        public uint uID;
        public uint uFlags;
        public uint uCallbackMessage;
        public IntPtr hIcon;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
        public string szTip;
        public uint dwState;
        public uint dwStateMask;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)]
        public string szInfo;
        public uint uTimeoutOrVersion;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)]
        public string szInfoTitle;
        public uint dwInfoFlags;
        public Guid guidItem;
        public IntPtr hBalloonIcon;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct POINT
    {
        public int X;
        public int Y;
    }

    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    private delegate IntPtr WindowProcDelegate(IntPtr windowHandle, uint message, UIntPtr wParam, IntPtr lParam);

    [DllImport("shell32.dll", EntryPoint = "Shell_NotifyIconW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ShellNotifyIcon(uint message, ref NOTIFYICONDATAW data);

    [DllImport("user32.dll", EntryPoint = "RegisterWindowMessageW", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern uint RegisterWindowMessage(string name);

    [DllImport("user32.dll", EntryPoint = "LoadIconW", SetLastError = true)]
    private static extern IntPtr LoadIcon(IntPtr instance, uint name);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr CopyIcon(IntPtr icon);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DestroyIcon(IntPtr icon);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongPtrW", SetLastError = true)]
    private static extern IntPtr SetWindowLongPtr(IntPtr windowHandle, int index, IntPtr newLong);

    [DllImport("user32.dll", EntryPoint = "CallWindowProcW")]
    private static extern IntPtr CallWindowProc(IntPtr previousWindowProc, IntPtr windowHandle, uint message, UIntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr CreatePopupMenu();

    [DllImport("user32.dll", EntryPoint = "AppendMenuW", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool AppendMenu(IntPtr menu, uint flags, uint identifier, string text);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DestroyMenu(IntPtr menu);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetCursorPos(out POINT point);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetForegroundWindow(IntPtr windowHandle);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint TrackPopupMenuEx(IntPtr menu, uint flags, int x, int y, IntPtr windowHandle, IntPtr parameters);

    [DllImport("user32.dll", EntryPoint = "PostMessageW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool PostMessage(IntPtr windowHandle, uint message, UIntPtr wParam, IntPtr lParam);
}
