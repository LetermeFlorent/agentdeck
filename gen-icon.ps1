Add-Type -AssemblyName System.Drawing
# PowerShell est insensible a la casse : on evite tout couple de noms qui ne differe
# que par la casse (ex. r/R) car ils designeraient la meme variable.
$N = 28
$cen = $N / 2.0
$SPOKES = 12
$RMAX = 13.0
$CORE = 2.4
$W0 = 0.42
$BASE = [System.Drawing.ColorTranslator]::FromHtml('#d97757')
$COREC = [System.Drawing.ColorTranslator]::FromHtml('#ecaf95')
$SIZE = 1024
$scale = $SIZE / $N
$ang = (2 * [Math]::PI) / $SPOKES

$bmp = New-Object System.Drawing.Bitmap($SIZE, $SIZE, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::Transparent)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
$brBase = New-Object System.Drawing.SolidBrush($BASE)
$brCore = New-Object System.Drawing.SolidBrush($COREC)

$cnt = 0
for ($y = 0; $y -lt $N; $y++) {
  for ($x = 0; $x -lt $N; $x++) {
    $dx = $x + 0.5 - $cen
    $dy = $y + 0.5 - $cen
    $rad = [Math]::Sqrt($dx * $dx + $dy * $dy)
    if ($rad -gt $RMAX) { continue }
    $th = [Math]::Atan2($dy, $dx)
    $dev = [Math]::Abs($th - [Math]::Round($th / $ang) * $ang)
    if ($rad -le $CORE -or $dev -le $W0 * (1 - $rad / $RMAX)) {
      $cnt++
      if ($rad -lt $CORE * 1.25) { $br = $brCore } else { $br = $brBase }
      $px = [int][Math]::Floor($x * $scale)
      $py = [int][Math]::Floor($y * $scale)
      $pw = [int][Math]::Ceiling($scale) + 1
      $g.FillRectangle($br, $px, $py, $pw, $pw)
    }
  }
}
$g.Dispose()
$bmp.Save('W:\agentdeck\icon-src.png', [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "cells=$cnt"
