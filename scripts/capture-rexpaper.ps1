# Captures the RexPaper window to a PNG and prints its path.
# Usage: powershell -File capture-rexpaper.ps1 [-OutFile path]
param(
  [string]$OutFile = "$env:TEMP\rexpaper-shot.png"
)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
[DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
public struct RECT { public int Left, Top, Right, Bottom; }
'@ -Name Win32Capture -Namespace RexPaper

$proc = $null
for ($i = 0; $i -lt 60; $i++) {
  $proc = Get-Process rexpaper -ErrorAction SilentlyContinue |
    Where-Object { $_.MainWindowHandle -ne 0 } |
    Select-Object -First 1
  if ($proc) { break }
  Start-Sleep -Seconds 1
}
if (-not $proc) { Write-Error "RexPaper window not found"; exit 1 }

$hwnd = $proc.MainWindowHandle
[RexPaper.Win32Capture]::ShowWindow($hwnd, 9) | Out-Null   # SW_RESTORE
[RexPaper.Win32Capture]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 600

$rect = New-Object RexPaper.Win32Capture+RECT
[RexPaper.Win32Capture]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top
if ($width -le 0 -or $height -le 0) { Write-Error "Bad window rect"; exit 1 }

$bmp = New-Object System.Drawing.Bitmap($width, $height)
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$gfx.CopyFromScreen($rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size($width, $height)))
$bmp.Save($OutFile, [System.Drawing.Imaging.ImageFormat]::Png)
$gfx.Dispose()
$bmp.Dispose()
Write-Output $OutFile
