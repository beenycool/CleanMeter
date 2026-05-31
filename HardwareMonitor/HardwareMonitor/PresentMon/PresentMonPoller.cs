using System.Diagnostics;
using System.Globalization;
using LibreHardwareMonitor.Hardware;
using Microsoft.Extensions.Logging;

// ReSharper disable FieldCanBeMadeReadOnly.Local
#pragma warning disable CS8618 // Non-nullable field must contain a non-null value when exiting constructor. Consider adding the 'required' modifier or declaring as nullable.

namespace HardwareMonitor.PresentMon;

public class PresentMonPoller(ILogger logger)
{
    private const string NO_SELECTED_APP = "NONE";

    private IHardware _hardware = new PresentMonHardware();
    public PresentMonSensor Displayed { get; private set; }
    public PresentMonSensor Presented { get; private set; }
    public PresentMonSensor Frametime { get; private set; }
    public HashSet<string> CurrentApps { get; private set; }

    public Action OnUpdateApps;

    private Process _process;
    private CultureInfo _cultureInfo = (CultureInfo)CultureInfo.CurrentCulture.Clone();

    private string _currentSelectedApp = NO_SELECTED_APP;

    public async Task Start(CancellationToken stoppingToken)
    {
        _cultureInfo.NumberFormat.NumberDecimalSeparator = ".";

        Displayed = new PresentMonSensor(_hardware, "displayed", 0, "Displayed Frames");
        Presented = new PresentMonSensor(_hardware, "presented", 1, "Presented Frames");
        Frametime = new PresentMonSensor(_hardware, "frametime", 2, "Frametime");
        CurrentApps = [];

        var ignoredProcessesPath = ResolveFilePath("ignored-processes.txt");
        string filteredApps = "";
        if (File.Exists(ignoredProcessesPath))
        {
            using var reader = new StreamReader(ignoredProcessesPath);
            var text = (await reader.ReadToEndAsync())
                .Split("\n", StringSplitOptions.RemoveEmptyEntries)
                .Select(x => $"--exclude {x.Trim()}");
            filteredApps = string.Join(" ", text);
        }
        else
        {
            logger.LogWarning("Ignored processes file not found at {Path}. No processes will be ignored.", ignoredProcessesPath);
        }

        try
        {
            await TerminateCurrentPresentMon();
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Failed to terminate existing PresentMon session. Continuing...");
        }

        var presentMonPath = ResolveFilePath("presentmon.exe");
        var processStartInfo = new ProcessStartInfo
        {
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            FileName = presentMonPath,
            Arguments =
                $"--stop_existing_session --no_console_stats --output_stdout --session_name HardwareMonitor {filteredApps}",
        };
        logger.LogInformation("Starting PresentMon process from {Path} with {Arguments}", presentMonPath, processStartInfo.Arguments);

        _process = new Process();
        _process.StartInfo = processStartInfo;
        _process.OutputDataReceived += (sender, args) => ParseData(args.Data);
        _process.ErrorDataReceived += (sender, args) => logger.LogError(args.Data);

        _process.Start();
        _process.BeginOutputReadLine();
        _process.BeginErrorReadLine();

        _ = ClearCurrentAppsAsync(stoppingToken);
        await _process.WaitForExitAsync(stoppingToken);
    }

    public void Stop()
    {
        _process.Kill(true);
    }

    private void ParseData(string? argsData)
    {
        string[] parts;
        if (argsData != null)
        {
            parts = argsData.Split(",");
            CurrentApps.Add(parts[0]);

            if (_currentSelectedApp != NO_SELECTED_APP && _currentSelectedApp != parts[0])
            {
                return;
            }

            if (float.TryParse(parts[9], NumberStyles.Any, _cultureInfo, out var frametime))
            {
                Frametime.Value = frametime;
            }

            if (float.TryParse(parts[13], NumberStyles.Any, _cultureInfo, out var gpuTime))
            {
                Presented.Value = gpuTime;
            }

            if (float.TryParse(parts[17], NumberStyles.Any, _cultureInfo, out var displayed))
            {
                Displayed.Value = displayed;
            }
        }
    }

    public void SetSelectedApp(string appName)
    {
        if (appName == "Auto")
        {
            _currentSelectedApp = NO_SELECTED_APP;
            return;
        }

        _currentSelectedApp = appName;
    }

    private async Task TerminateCurrentPresentMon()
    {
        var presentMonPath = ResolveFilePath("presentmon.exe");
        var processStartInfo = new ProcessStartInfo
        {
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            FileName = presentMonPath,
            Arguments =
                $"--terminate_existing_session --no_console_stats --output_stdout --session_name HardwareMonitor",
        };
        logger.LogInformation("Starting PresentMon terminate from {Path} with {Arguments}", presentMonPath, processStartInfo.Arguments);

        var process = new Process();
        process.StartInfo = processStartInfo;
        process.Start();
        await process.WaitForExitAsync();
    }

    private async Task ClearCurrentAppsAsync(CancellationToken cancellationToken)
    {
        if (cancellationToken.IsCancellationRequested) return;
        await Task.Delay(10_000, cancellationToken);
        OnUpdateApps?.Invoke();
        CurrentApps.Clear();
        _ = ClearCurrentAppsAsync(cancellationToken);
    }

    private string ResolveFilePath(string filename)
    {
        string baseDir = AppDomain.CurrentDomain.BaseDirectory;

        string[] searchPaths =
        [
            baseDir,
            Path.Combine(baseDir, "_up_"),
            Path.Combine(baseDir, "resources"),
        ];

        string? parentDir = Path.GetDirectoryName(baseDir.TrimEnd(Path.DirectorySeparatorChar));
        if (!string.IsNullOrEmpty(parentDir))
        {
            searchPaths =
            [
                ..searchPaths,
                parentDir,
                Path.Combine(parentDir, "_up_"),
                Path.Combine(parentDir, "resources"),
            ];
        }

        foreach (var dir in searchPaths)
        {
            var path = Path.Combine(dir, filename);
            if (File.Exists(path))
            {
                logger.LogInformation("Resolved {Filename} at {Path}", filename, path);
                return path;
            }
        }

        logger.LogWarning("Could not find {Filename} in any search path. Falling back to bare filename.", filename);
        return filename;
    }
}