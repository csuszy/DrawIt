# DrawIt Automated Packaging Script
# Generates:
#   1. DrawIt.msix       (Modern Windows App / Microsoft Store package)
# Note: Inno Setup installer and standalone zip are retired in favor of Portable, MSIX and MS Store Installer.

param(
    [switch]$SkipReleaseBuild = $false,
    [switch]$SignMsix = $true
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
Set-Location $RootDir

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   DrawIt Packaging Pipeline (v0.3.0)   " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Build release binary if requested
if (-not $SkipReleaseBuild) {
    Write-Host "[1/4] Building release binary with Cargo..." -ForegroundColor Yellow
    cargo build --release
    Copy-Item "target\release\drawit.exe" "DrawIt.exe" -Force
} else {
    Write-Host "[1/4] Skipping cargo build (using existing DrawIt.exe)..." -ForegroundColor Gray
}

# 2. Verify / Generate MSIX Assets
Write-Host "[2/4] Verifying MSIX visual assets..." -ForegroundColor Yellow
$AssetsDir = Join-Path $ScriptDir "msix\Assets"
if (-not (Test-Path $AssetsDir)) {
    New-Item -ItemType Directory -Force -Path $AssetsDir | Out-Null
}

Add-Type -AssemblyName System.Drawing, PresentationCore, WindowsBase
$icoPath = Join-Path $RootDir "DrawIt_logo.ico"
$stream = [System.IO.File]::OpenRead($icoPath)
$decoder = [System.Windows.Media.Imaging.IconBitmapDecoder]::new($stream, [System.Windows.Media.Imaging.BitmapCreateOptions]::PreservePixelFormat, [System.Windows.Media.Imaging.BitmapCacheOption]::Default)
$frame256 = $decoder.Frames | Where-Object { $_.PixelWidth -eq 256 } | Select-Object -First 1

$enc = [System.Windows.Media.Imaging.PngBitmapEncoder]::new()
$enc.Frames.Add($frame256)
$ms = [System.IO.MemoryStream]::new()
$enc.Save($ms)
$stream.Close()
$ms.Position = 0
$srcBmp = [System.Drawing.Bitmap]::FromStream($ms)

function Create-Asset($filename, $canvasW, $canvasH, $iconW, $iconH) {
    $bmp = New-Object System.Drawing.Bitmap($canvasW, $canvasH, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)
    
    $x = [Math]::Round(($canvasW - $iconW) / 2)
    $y = [Math]::Round(($canvasH - $iconH) / 2)
    $g.DrawImage($srcBmp, [int]$x, [int]$y, [int]$iconW, [int]$iconH)
    $g.Dispose()
    
    $outPath = Join-Path $AssetsDir $filename
    $bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

Create-Asset "Square44x44Logo.png" 44 44 44 44
Create-Asset "Square44x44Logo.targetsize-24_altform-unplated.png" 24 24 24 24
Create-Asset "Square44x44Logo.targetsize-48_altform-unplated.png" 48 48 48 48
Create-Asset "Square44x44Logo.targetsize-256_altform-unplated.png" 256 256 256 256
Create-Asset "Square71x71Logo.png" 71 71 71 71
Create-Asset "Square150x150Logo.png" 150 150 150 150
Create-Asset "Wide310x150Logo.png" 310 150 130 130
Create-Asset "StoreLogo.png" 50 50 50 50
Create-Asset "SplashScreen.png" 620 300 180 180
$srcBmp.Dispose()
$ms.Dispose()

# 3. Build MSIX Package
Write-Host "[3/4] Packaging MSIX (MakeAppx)..." -ForegroundColor Yellow
$LayoutDir = Join-Path $RootDir "target\msix_layout"
if (Test-Path $LayoutDir) { Remove-Item -Recurse -Force $LayoutDir }
New-Item -ItemType Directory -Force -Path (Join-Path $LayoutDir "Assets") | Out-Null
Copy-Item "DrawIt.exe" (Join-Path $LayoutDir "DrawIt.exe") -Force
Copy-Item (Join-Path $ScriptDir "msix\AppxManifest.xml") (Join-Path $LayoutDir "AppxManifest.xml") -Force
Copy-Item (Join-Path $AssetsDir "*") (Join-Path $LayoutDir "Assets\") -Force

$MakeAppxCandidates = @(
    "C:\Users\bcsas\AppData\Local\Microsoft\WinGet\Packages\Microsoft.MSIX-Toolkit_Microsoft.Winget.Source_8wekyb3d8bbwe\MSIX-Toolkit.x64\MakeAppx.exe",
    "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\makeappx.exe",
    "C:\Program Files (x86)\Windows Kits\10\bin\x64\makeappx.exe"
)
$MakeAppx = $MakeAppxCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($MakeAppx) {
    $msixPath = Join-Path $RootDir "DrawIt.msix"
    & $MakeAppx pack /d $LayoutDir /p $msixPath /o /nv | Out-Null
    Write-Host "   Created: $msixPath" -ForegroundColor Green

    if ($SignMsix) {
        $SignToolCandidates = @(
            "C:\Users\bcsas\AppData\Local\Microsoft\WinGet\Packages\Microsoft.MSIX-Toolkit_Microsoft.Winget.Source_8wekyb3d8bbwe\MSIX-Toolkit.x64\signtool.exe",
            "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe"
        )
        $SignTool = $SignToolCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
        if ($SignTool) {
            Write-Host "   Signing MSIX with developer certificate..." -ForegroundColor Gray
            $pfxPath = Join-Path $RootDir "target\DrawIt_Dev.pfx"
            if (-not (Test-Path $pfxPath)) {
                $cert = New-SelfSignedCertificate -Type Custom -Subject "CN=0D299B98-5051-435C-8BB5-970045368EEC" -KeyUsage DigitalSignature -FriendlyName "DrawIt Dev" -CertStoreLocation "Cert:\CurrentUser\My" -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3")
                $pwd = ConvertTo-SecureString -String "DrawItDev123" -Force -AsPlainText
                Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $pwd | Out-Null
            }
            & $SignTool sign /fd SHA256 /a /f $pfxPath /p "DrawItDev123" $msixPath | Out-Null
            Write-Host "   Signed: $msixPath" -ForegroundColor Green
        }
    }
} else {
    Write-Warning "MakeAppx.exe not found. MSIX creation skipped."
}

# 4. Inno Setup installer skipped (only Portable, MSIX and MS Store Installer released)
Write-Host "[4/4] Inno Setup installer skipped (releases are Portable and MSIX only)." -ForegroundColor Gray

Write-Host "`nAll packages built successfully!" -ForegroundColor Cyan
