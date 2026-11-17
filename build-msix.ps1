# DrawIt - Microsoft Store MSIX Build Script
# Forditas a 'store' feature-rel es kozvetlenul feltoltheto MSIX csomag keszitese

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   DrawIt Microsoft Store MSIX Builder    " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Forditas Cargo-val (store feature aktiv, kulso GitHub frissito kizarva)
Write-Host "`n[1/4] Forditas 'store' feature-rel (cargo build --release --features store)..." -ForegroundColor Yellow
cargo build --release --features store
if ($LASTEXITCODE -ne 0) {
    Write-Error "A Cargo forditas sikertelen!"
    exit 1
}

# 2. Cel- es ideiglenes konyvtarak elokeszitese
Write-Host "[2/4] Csomagolasi konyvtar osszeallitasa es kepek ellenorzese..." -ForegroundColor Yellow
$DistDir = "dist"
$StagingDir = "target\msix_staging"
$AssetsDir = "msix\Assets"

if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
}

if (-not (Test-Path $AssetsDir)) {
    New-Item -ItemType Directory -Force -Path $AssetsDir | Out-Null
}

# Szukseges kepek megletenek ellenorzese
Add-Type -AssemblyName System.Drawing
$requiredImages = @(
    @{ Name = "StoreLogo.png"; Width = 50; Height = 50; IconW = 50; IconH = 50 },
    @{ Name = "Square44x44Logo.png"; Width = 44; Height = 44; IconW = 44; IconH = 44 },
    @{ Name = "Square150x150Logo.png"; Width = 150; Height = 150; IconW = 150; IconH = 150 },
    @{ Name = "Square71x71Logo.png"; Width = 71; Height = 71; IconW = 71; IconH = 71 },
    @{ Name = "Wide310x150Logo.png"; Width = 310; Height = 150; IconW = 130; IconH = 130 },
    @{ Name = "SplashScreen.png"; Width = 620; Height = 300; IconW = 180; IconH = 180 }
)

$srcLogo = "img\DrawIt_logo.png"
if (Test-Path $srcLogo) {
    $srcImg = [System.Drawing.Image]::FromFile((Resolve-Path $srcLogo))
    foreach ($imgDef in $requiredImages) {
        $imgPath = Join-Path $AssetsDir $imgDef.Name
        if (-not (Test-Path $imgPath)) {
            Write-Host "   Kep generalasa: $($imgDef.Name) ($($imgDef.Width)x$($imgDef.Height))..." -ForegroundColor DarkGray
            $bmp = New-Object System.Drawing.Bitmap($imgDef.Width, $imgDef.Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
            $g = [System.Drawing.Graphics]::FromImage($bmp)
            $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
            $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
            $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
            $g.Clear([System.Drawing.Color]::Transparent)
            $x = [Math]::Round(($imgDef.Width - $imgDef.IconW) / 2)
            $y = [Math]::Round(($imgDef.Height - $imgDef.IconH) / 2)
            $g.DrawImage($srcImg, [int]$x, [int]$y, [int]$imgDef.IconW, [int]$imgDef.IconH)
            $g.Dispose()
            $bmp.Save($imgPath, [System.Drawing.Imaging.ImageFormat]::Png)
            $bmp.Dispose()
        }
    }
    $srcImg.Dispose()
}

if (Test-Path $StagingDir) {
    Remove-Item -Recurse -Force $StagingDir
}
New-Item -ItemType Directory -Force -Path "$StagingDir\Assets" | Out-Null

# Fajlok atmasolasa a csomagolasi konyvtarba
Copy-Item "target\release\drawit.exe" "$StagingDir\DrawIt.exe" -Force
Copy-Item "$AssetsDir\*" "$StagingDir\Assets\" -Force

# Csomagolando kepek megletenek es mereteinek teteles ellenorzese
Write-Host "   Csomagolando Assets mappa ellenorzese:" -ForegroundColor Cyan
foreach ($imgDef in $requiredImages) {
    $stagedImg = Join-Path "$StagingDir\Assets" $imgDef.Name
    if (-not (Test-Path $stagedImg)) {
        Write-Error "HIANYZO KEPFAJL: $stagedImg"
        exit 1
    }
    $imgTest = [System.Drawing.Image]::FromFile($stagedImg)
    Write-Host "   [OK] $($imgDef.Name): $($imgTest.Width)x$($imgTest.Height) pixel" -ForegroundColor Green
    $imgTest.Dispose()
}

# Manifest masolasa es ellenorzese
$manifestContent = Get-Content "msix\AppxManifest.xml" -Raw
# Biztositjuk a Partner Center azonositoit
if ($manifestContent -match '\[IDE_IRD_A_PUBLISHER_DISPLAY_NAME_ERTEKET\]') {
    $manifestContent = $manifestContent -replace '\[IDE_IRD_A_PUBLISHER_DISPLAY_NAME_ERTEKET\]', 'csuszy'
}
if ($manifestContent -match '\[IDE_IRD_A_PACKAGE_IDENTITY_NAME_ERTEKET\]') {
    $manifestContent = $manifestContent -replace '\[IDE_IRD_A_PACKAGE_IDENTITY_NAME_ERTEKET\]', 'csuszy.DrawIt-DrawonScreen'
    $manifestContent = $manifestContent -replace 'CN=\[IDE_IRD_A_PUBLISHER_CN_ERTEKET\]', 'CN=0D299B98-5051-435C-8BB5-970045368EEC'
}
Set-Content -Path "$StagingDir\AppxManifest.xml" -Value $manifestContent -Encoding UTF8

# 3. Legfrissebb makeappx.exe megkeresese (Windows Kits alatt, szukseg eseten fallback)
Write-Host "`n[3/4] MakeAppx.exe keresese a rendszeren..." -ForegroundColor Yellow

$makeappxCandidates = @()

if (Test-Path "C:\Program Files (x86)\Windows Kits\10\bin") {
    $foundKits = Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits\10\bin" -Filter "makeappx.exe" -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -like "*x64*" } |
        Sort-Object { $_.Directory.Name } -Descending |
        Select-Object -ExpandProperty FullName
    if ($foundKits) { $makeappxCandidates += $foundKits }
}

if (Test-Path "C:\Program Files\Windows Kits\10\bin") {
    $foundKits64 = Get-ChildItem -Path "C:\Program Files\Windows Kits\10\bin" -Filter "makeappx.exe" -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -like "*x64*" } |
        Sort-Object { $_.Directory.Name } -Descending |
        Select-Object -ExpandProperty FullName
    if ($foundKits64) { $makeappxCandidates += $foundKits64 }
}

# WinGet MSIX-Toolkit es PATH fallback
$wingetMsix = Get-ChildItem -Path "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Filter "MakeAppx.exe" -Recurse -ErrorAction SilentlyContinue |
    Select-Object -First 1 -ExpandProperty FullName
if ($wingetMsix) { $makeappxCandidates += $wingetMsix }

$cmdMsix = (Get-Command "makeappx.exe" -ErrorAction SilentlyContinue).Source
if ($cmdMsix) { $makeappxCandidates += $cmdMsix }

$makeappx = $makeappxCandidates | Select-Object -First 1

if (-not $makeappx) {
    Write-Error "Nem talalhato a MakeAppx.exe a rendszeren! Telepisd a Windows 10/11 SDK-t vagy az MSIX Toolkittet."
    exit 1
}

Write-Host "   MakeAppx eszkoz: $makeappx" -ForegroundColor DarkGray

# 4. MSIX csomag eloallitasa (alairas nelkul, Store kompatibilis)
Write-Host "`n[4/4] DrawIt.msix csomagolasa..." -ForegroundColor Yellow
$outMsixDist = Join-Path $DistDir "DrawIt.msix"
$outMsixRoot = "DrawIt.msix"

if (Test-Path $outMsixDist) { Remove-Item -Force $outMsixDist }
if (Test-Path $outMsixRoot) { Remove-Item -Force $outMsixRoot }

& $makeappx pack /d $StagingDir /p $outMsixDist /o
if ($LASTEXITCODE -ne 0) {
    Write-Error "A MakeAppx csomagolas meghisult!"
    exit 1
}

# Masolas a gyokerbe is, hogy mindket helyrol elerheto legyen
Copy-Item $outMsixDist $outMsixRoot -Force

$msixItem = Get-Item $outMsixDist
Write-Host "`n==========================================" -ForegroundColor Green
Write-Host "Sikeres MSIX csomagolas!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host "Letrejott: $($msixItem.FullName)" -ForegroundColor Green
Write-Host "Masolva:   $(Resolve-Path $outMsixRoot)" -ForegroundColor Green
Write-Host "Meret:     $([Math]::Round($msixItem.Length / 1KB, 1)) KB" -ForegroundColor Green
Write-Host "`nManifest ellenorzes:" -ForegroundColor Cyan
Write-Host "  - Package Name:         csuszy.DrawIt-DrawonScreen" -ForegroundColor Gray
Write-Host "  - Package Publisher:    CN=0D299B98-5051-435C-8BB5-970045368EEC" -ForegroundColor Gray
Write-Host "  - DisplayName:          DrawIt - Draw on Screen" -ForegroundColor Gray
Write-Host "  - PublisherDisplayName: csuszy" -ForegroundColor Gray
Write-Host "  - Resource Language:    en-us" -ForegroundColor Gray
Write-Host "  - Store Product ID:     9NHQC57FT5LQ" -ForegroundColor Gray
Write-Host "  - Ikonok:               StoreLogo.png (50x50), Square44x44Logo.png (44x44), Square150x150Logo.png (150x150)" -ForegroundColor Gray
