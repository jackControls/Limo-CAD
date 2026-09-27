// QA-only Windows IME prerequisite probe. Never sends Unicode or IME messages.
// COM layouts/order match the Windows SDK msctf.h; no product dependencies.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

public static class WindowsImeProbe {
    [StructLayout(LayoutKind.Sequential)] public struct Profile {
        public uint Type; public ushort Language; public Guid ClassId, ProfileId, Category;
        public IntPtr Substitute; public uint Capabilities; public IntPtr Layout; public uint Flags;
    }
    [ComImport, Guid("71c6e74d-0f28-11d8-a82a-00065b84435c"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface Profiles {
        [PreserveSig] int Clone(out Profiles copy);
        [PreserveSig] int Next(uint count, out Profile profile, out uint fetched);
        [PreserveSig] int Reset();
        [PreserveSig] int Skip(uint count);
    }
    [ComImport, Guid("71c6e74c-0f28-11d8-a82a-00065b84435c"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface ProfileManager {
        [PreserveSig] int Activate(uint type, ushort language, ref Guid clsid, ref Guid profile, IntPtr layout, uint flags);
        [PreserveSig] int Deactivate(uint type, ushort language, ref Guid clsid, ref Guid profile, IntPtr layout, uint flags);
        [PreserveSig] int Get(uint type, ushort language, ref Guid clsid, ref Guid profile, IntPtr layout, out Profile value);
        [PreserveSig] int Enumerate(ushort language, out Profiles profiles);
        [PreserveSig] int Release(ref Guid clsid, uint flags);
        [PreserveSig] int Register(ref Guid clsid, ushort language, ref Guid profile, IntPtr description, uint descriptionLength,
            IntPtr icon, uint iconLength, uint index, IntPtr substitute, uint preferred, int enabled, uint flags);
        [PreserveSig] int Unregister(ref Guid clsid, ushort language, ref Guid profile, uint flags);
        [PreserveSig] int Active(ref Guid category, out Profile profile);
    }
    // ITfInputProcessorProfiles, in Windows SDK vtable order. Only the current
    // user's existing Microsoft Japanese profile is enabled; no default-user API.
    [ComImport, Guid("1f02b6c5-7842-4ee6-8a0b-9a24183a95ca"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface UserProfiles {
        [PreserveSig] int Register(ref Guid clsid);
        [PreserveSig] int Unregister(ref Guid clsid);
        [PreserveSig] int Add(ref Guid clsid, ushort language, ref Guid profile, IntPtr description, uint descriptionLength, IntPtr icon, uint iconLength, uint iconIndex);
        [PreserveSig] int Remove(ref Guid clsid, ushort language, ref Guid profile);
        [PreserveSig] int EnumerateProcessors(out IntPtr enumerator);
        [PreserveSig] int GetDefault(ushort language, ref Guid category, out Guid clsid, out Guid profile);
        [PreserveSig] int SetDefault(ushort language, ref Guid clsid, ref Guid profile);
        [PreserveSig] int Activate(ref Guid clsid, ushort language, ref Guid profile);
        [PreserveSig] int GetActive(ref Guid clsid, out ushort language, out Guid profile);
        [PreserveSig] int Description(ref Guid clsid, ushort language, ref Guid profile, out IntPtr description);
        [PreserveSig] int CurrentLanguage(out ushort language);
        [PreserveSig] int ChangeLanguage(ushort language);
        [PreserveSig] int LanguageList(out IntPtr languages, out uint count);
        [PreserveSig] int EnumerateLanguage(ushort language, out IntPtr enumerator);
        [PreserveSig] int Enable(ref Guid clsid, ushort language, ref Guid profile, int enabled);
        [PreserveSig] int IsEnabled(ref Guid clsid, ushort language, ref Guid profile, out int enabled);
    }
    static void RequireDisposableRunner() {
        if (Environment.GetEnvironmentVariable("GITHUB_ACTIONS") != "true"
            || Environment.GetEnvironmentVariable("RUNNER_OS") != "Windows"
            || Environment.GetEnvironmentVariable("RUNNER_ENVIRONMENT") != "github-hosted"
            || Environment.GetEnvironmentVariable("GITHUB_REPOSITORY") != "jackControls/noBS-CAD"
            || !System.Text.RegularExpressions.Regex.IsMatch(Environment.GetEnvironmentVariable("GITHUB_RUN_ID") ?? "", @"^\d+$"))
            throw new InvalidOperationException("Profile changes and input require the disposable GitHub-hosted noBS-CAD Windows job");
    }
    public static object EnableJapaneseProfile() {
        RequireDisposableRunner();
        var manager = (UserProfiles)Activator.CreateInstance(Type.GetTypeFromCLSID(new Guid("33c53a50-f456-4884-b049-85fd643ecfed")));
        var clsid = new Guid("03b5835f-f03c-411b-9ce2-aa23e1171e36");
        var profile = new Guid("a76c93d9-5523-4e90-aafa-4db112f9ac76");
        try {
            int before, after;
            Marshal.ThrowExceptionForHR(manager.IsEnabled(ref clsid, 0x411, ref profile, out before));
            int result = before != 0 ? 0 : manager.Enable(ref clsid, 0x411, ref profile, 1);
            Marshal.ThrowExceptionForHR(result);
            Marshal.ThrowExceptionForHR(manager.IsEnabled(ref clsid, 0x411, ref profile, out after));
            if (after == 0) throw new InvalidOperationException("Microsoft Japanese profile remained disabled after EnableLanguageProfile");
            return new Dictionary<string, object> {
                {"api", "ITfInputProcessorProfiles::EnableLanguageProfile"}, {"scope", "current disposable user"},
                {"enabled_before", before != 0}, {"enabled_after", after != 0}, {"hresult", result.ToString("X8")}
            };
        } finally { Marshal.ReleaseComObject(manager); }
    }
    static ProfileManager Manager() {
        return (ProfileManager)Activator.CreateInstance(Type.GetTypeFromCLSID(new Guid("33c53a50-f456-4884-b049-85fd643ecfed")));
    }
    static Dictionary<string, object> Describe(Profile p) {
        return new Dictionary<string, object> {
            {"type", p.Type}, {"language", p.Language.ToString("X4")}, {"class_id", p.ClassId.ToString()},
            {"profile_id", p.ProfileId.ToString()}, {"category", p.Category.ToString()},
            {"enabled", (p.Flags & 2) != 0}, {"active", (p.Flags & 1) != 0}, {"flags", p.Flags},
            {"layout", p.Layout.ToInt64().ToString("X")}, {"capabilities", p.Capabilities}
        };
    }
    public static object[] EnumerateProfiles() {
        var result = new List<object>(); var manager = Manager(); Profiles profiles = null;
        try {
            Marshal.ThrowExceptionForHR(manager.Enumerate(0, out profiles));
            for (int i = 0; i < 256; i++) {
                Profile profile; uint fetched;
                int hr = profiles.Next(1, out profile, out fetched);
                Marshal.ThrowExceptionForHR(hr);
                if (fetched == 0) return result.ToArray();
                result.Add(Describe(profile));
            }
            throw new InvalidOperationException("Unexpectedly more than 256 TSF profiles");
        } finally {
            if (profiles != null) Marshal.ReleaseComObject(profiles);
            Marshal.ReleaseComObject(manager);
        }
    }
    static Profile ActiveProfile() {
        var manager = Manager();
        try {
            var category = new Guid("34745c63-b2f0-4784-8b67-5e12c8701a31"); Profile result;
            Marshal.ThrowExceptionForHR(manager.Active(ref category, out result)); return result;
        } finally { Marshal.ReleaseComObject(manager); }
    }
    static bool Japanese(Profile profile) {
        return profile.Type == 1 && profile.Language == 0x411
            && profile.ClassId == new Guid("03b5835f-f03c-411b-9ce2-aa23e1171e36")
            && profile.ProfileId == new Guid("a76c93d9-5523-4e90-aafa-4db112f9ac76");
    }
    [DllImport("user32.dll")] static extern IntPtr GetProcessWindowStation();
    [DllImport("user32.dll")] static extern IntPtr GetThreadDesktop(uint thread);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool GetUserObjectInformation(IntPtr handle, int index, StringBuilder value, uint length, out uint needed);
    static string ObjectName(IntPtr handle) {
        var name = new StringBuilder(256); uint needed;
        if (!GetUserObjectInformation(handle, 2, name, 512, out needed))
            throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        return name.ToString();
    }
    public static object Desktop() {
        return new Dictionary<string, object> {
            {"station", ObjectName(GetProcessWindowStation())},
            {"desktop", ObjectName(GetThreadDesktop(GetCurrentThreadId()))},
            {"session_id", Process.GetCurrentProcess().SessionId}, {"user_interactive", Environment.UserInteractive}
        };
    }

    [StructLayout(LayoutKind.Sequential)] struct KeyboardInput {
        public ushort Key, Scan; public uint Flags, Time; public UIntPtr Extra;
    }
    [StructLayout(LayoutKind.Explicit, Size = 32)] struct InputUnion { [FieldOffset(0)] public KeyboardInput Keyboard; }
    [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public InputUnion Data; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint count, Input[] input, int size);
    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern IntPtr GetFocus();
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("imm32.dll")] static extern IntPtr ImmGetContext(IntPtr window);
    [DllImport("imm32.dll")] static extern bool ImmReleaseContext(IntPtr window, IntPtr context);
    [DllImport("imm32.dll", CharSet = CharSet.Unicode)] static extern int ImmGetCompositionStringW(IntPtr context, uint kind, byte[] value, uint length);
    static void Key(ushort key, bool up) {
        var input = new Input { Type = 1, Data = new InputUnion { Keyboard = new KeyboardInput { Key = key, Flags = up ? 2u : 0u } } };
        if (SendInput(1, new[] { input }, Marshal.SizeOf(typeof(Input))) != 1)
            throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error(), "SendInput failed");
    }
    static string Composition(IntPtr context, uint kind) {
        int length = ImmGetCompositionStringW(context, kind, null, 0);
        if (length < 0) throw new InvalidOperationException("ImmGetCompositionStringW returned " + length);
        if (length == 0) return "";
        if (length > 8192) throw new InvalidOperationException("Unexpected composition length");
        var bytes = new byte[length];
        int read = ImmGetCompositionStringW(context, kind, bytes, (uint)length);
        if (read < 0) throw new InvalidOperationException("ImmGetCompositionStringW read returned " + read);
        return Encoding.Unicode.GetString(bytes, 0, read);
    }
    sealed class ObservedTextBox : TextBox {
        public readonly List<object> ImeEvents = new List<object>();
        public string LastPreedit = "", Result = "", Error;
        public int Starts, Ends, Results;
        protected override void WndProc(ref Message message) {
            try {
                if (message.Msg == 0x10d) { Starts++; ImeEvents.Add(new { message = "WM_IME_STARTCOMPOSITION" }); }
                if (message.Msg == 0x10e) { Ends++; ImeEvents.Add(new { message = "WM_IME_ENDCOMPOSITION" }); }
                if (message.Msg == 0x10f) {
                    uint flags = unchecked((uint)message.LParam.ToInt64());
                    IntPtr context = ImmGetContext(Handle);
                    try {
                        if ((flags & 8) != 0) {
                            LastPreedit = Composition(context, 8);
                            ImeEvents.Add(new { message = "WM_IME_COMPOSITION", flags = flags, preedit = LastPreedit });
                        }
                        if ((flags & 0x800) != 0) {
                            Result = Composition(context, 0x800); Results++;
                            ImeEvents.Add(new { message = "WM_IME_COMPOSITION", flags = flags, result = Result });
                        }
                    } finally { ImmReleaseContext(Handle, context); }
                }
            } catch (Exception ex) { Error = ex.ToString(); }
            base.WndProc(ref message);
        }
    }
    // Called only by the guarded CI script. This is an owned stock textbox,
    // not the Bevy host; success means environment feasibility only.
    public static object Exercise() {
        RequireDisposableRunner();
        if (System.Threading.Thread.CurrentThread.GetApartmentState() != System.Threading.ApartmentState.STA)
            throw new InvalidOperationException("Run the probe in STA Windows PowerShell");
        if (IntPtr.Size != 8) throw new InvalidOperationException("The probe targets the x64 hosted runner");
        var report = new Dictionary<string, object> {
            {"status", "failed"}, {"event_source", "ordinary virtual-key SendInput through Windows Microsoft Japanese IME"},
            {"native_bevy_validated", false}, {"candidate_placement", "not tested"}, {"popup_pixels", "not captured"}
        };
        using (var form = new Form()) using (var timer = new Timer()) {
            form.Text = "noBS CAD disposable IME prerequisite probe";
            form.Width = 620; form.Height = 180; form.StartPosition = FormStartPosition.CenterScreen; form.TopMost = true;
            var field = new ObservedTextBox { Left = 24, Top = 40, Width = 540, ImeMode = ImeMode.On };
            form.Controls.Add(field);
            var keys = new List<object>(); var profiles = new List<object>();
            var watch = Stopwatch.StartNew(); int stage = 0, switches = 0;
            Action requireFocus = () => {
                if (GetForegroundWindow() != form.Handle || GetFocus() != field.Handle)
                    throw new InvalidOperationException("Owned probe field lost foreground/focus; no further keys sent");
            };
            Action<ushort, ushort> chord = (modifier, key) => {
                requireFocus(); keys.Add(new { modifier = modifier, key = key, elapsed_ms = watch.ElapsedMilliseconds });
                if (modifier != 0) Key(modifier, false);
                try { Key(key, false); Key(key, true); }
                finally { if (modifier != 0) Key(modifier, true); }
            };
            Action typeHaru = () => { foreach (ushort key in new ushort[] { 0x48, 0x41, 0x52, 0x55 }) chord(0, key); };
            timer.Interval = 400;
            timer.Tick += (sender, args) => {
                try {
                    if (watch.ElapsedMilliseconds > 20000) throw new TimeoutException("IME feasibility probe timed out at stage " + stage);
                    if (field.Error != null) throw new InvalidOperationException(field.Error);
                    requireFocus();
                    if (stage == 0) {
                        Profile active = ActiveProfile(); profiles.Add(Describe(active));
                        if (!Japanese(active)) {
                            if (++switches > 8) throw new InvalidOperationException("Microsoft Japanese IME did not activate after eight Win+Space switches");
                            chord(0x5b, 0x20); return;
                        }
                        report["active_profile"] = Describe(active); chord(0x11, 0x14); stage = 1;
                    } else if (stage == 1) { typeHaru(); stage = 2; }
                    else if (stage == 2 && field.LastPreedit == "\u306f\u308b" && field.Starts > 0) {
                        report["preedit"] = field.LastPreedit; report["results_before_commit"] = field.Results;
                        if (field.Results != 0) throw new InvalidOperationException("IME committed before Enter");
                        chord(0, 0x0d); stage = 3;
                    } else if (stage == 3 && field.Result == "\u306f\u308b" && field.Text == "\u306f\u308b") {
                        report["committed"] = field.Text; field.LastPreedit = ""; typeHaru(); stage = 4;
                    } else if (stage == 4 && field.LastPreedit == "\u306f\u308b" && field.Starts >= 2) {
                        chord(0, 0x1b); stage = 5;
                    } else if (stage == 5 && field.Ends >= 2 && field.Text == "\u306f\u308b") {
                        if (field.Results != 1) throw new InvalidOperationException("Composition did not commit exactly once");
                        report["cancelled_text"] = field.Text; report["status"] = "stock-control-ime-feasible";
                        timer.Stop(); form.Close();
                    }
                } catch (Exception ex) {
                    report["error"] = ex.ToString(); report["hresult"] = ex.HResult.ToString("X8");
                    timer.Stop(); form.Close();
                }
            };
            form.Shown += (sender, args) => {
                report["pid"] = Process.GetCurrentProcess().Id; report["window"] = form.Handle.ToInt64();
                report["field_window"] = field.Handle.ToInt64();
                SetForegroundWindow(form.Handle); field.Focus(); timer.Start();
            };
            form.FormClosing += (sender, args) => { report["final_text"] = field.Text; };
            Application.Run(form);
            report["elapsed_ms"] = watch.ElapsedMilliseconds; report["keys"] = keys;
            report["profiles_observed_on_ui_thread"] = profiles; report["received_ime_messages"] = field.ImeEvents;
        }
        return report;
    }
}
