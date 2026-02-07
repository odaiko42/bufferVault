# BufferVault - Generateur d'icone
# Cree un fichier .ico avec un bouclier et les lettres "BV"
# Tailles : 16, 32, 48, 256

Add-Type -AssemblyName System.Drawing

function New-ShieldBitmap {
    param([int]$Size)

    $bmp = New-Object System.Drawing.Bitmap($Size, $Size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
    $g.Clear([System.Drawing.Color]::Transparent)

    # Marge
    $m = [Math]::Max(1, [int]($Size * 0.06))

    # Points du bouclier (haut plat, bas pointu)
    $pts = @(
        (New-Object System.Drawing.PointF($m, $m)),
        (New-Object System.Drawing.PointF(($Size - $m), $m)),
        (New-Object System.Drawing.PointF(($Size - $m), ($Size * 0.58))),
        (New-Object System.Drawing.PointF(($Size / 2), ($Size - $m))),
        (New-Object System.Drawing.PointF($m, ($Size * 0.58)))
    )

    # Fond du bouclier : bleu fonce
    $shieldBrush = New-Object System.Drawing.SolidBrush(
        [System.Drawing.Color]::FromArgb(255, 20, 24, 58))
    $g.FillPolygon($shieldBrush, $pts)

    # Bordure bleue
    $penW = [Math]::Max(1, [int]($Size / 14))
    $borderPen = New-Object System.Drawing.Pen(
        [System.Drawing.Color]::FromArgb(255, 74, 158, 255), $penW)
    $g.DrawPolygon($borderPen, $pts)

    # Barre horizontale decorative (style coffre-fort)
    $barY = [int]($Size * 0.32)
    $barH = [Math]::Max(1, [int]($Size * 0.03))
    $barBrush = New-Object System.Drawing.SolidBrush(
        [System.Drawing.Color]::FromArgb(120, 74, 158, 255))
    $g.FillRectangle($barBrush, ($m + $penW), $barY, ($Size - 2*$m - 2*$penW), $barH)

    # Texte "BV"
    $fontSize = [Math]::Max(5, $Size * 0.30)
    $font = New-Object System.Drawing.Font("Segoe UI", $fontSize, [System.Drawing.FontStyle]::Bold)
    $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $textRect = New-Object System.Drawing.RectangleF(0, ($Size * 0.05), $Size, ($Size * 0.85))
    $g.DrawString("BV", $font, $textBrush, $textRect, $sf)

    # Petit verrou sous le texte
    $lockSize = [Math]::Max(2, [int]($Size * 0.12))
    $lockX = [int](($Size - $lockSize) / 2)
    $lockY = [int]($Size * 0.62)
    $lockBrush = New-Object System.Drawing.SolidBrush(
        [System.Drawing.Color]::FromArgb(200, 255, 185, 0))
    if ($Size -ge 32) {
        $g.FillRectangle($lockBrush, $lockX, $lockY, $lockSize, [int]($lockSize * 0.8))
        $arcPen = New-Object System.Drawing.Pen(
            [System.Drawing.Color]::FromArgb(200, 255, 185, 0), [Math]::Max(1, [int]($Size / 20)))
        $arcW = [int]($lockSize * 0.6)
        $arcH = [int]($lockSize * 0.5)
        $arcX = [int]($lockX + ($lockSize - $arcW) / 2)
        $arcY = [int]($lockY - $arcH + 1)
        $g.DrawArc($arcPen, $arcX, $arcY, $arcW, $arcH, 180, 180)
    }

    $g.Dispose()
    $shieldBrush.Dispose()
    $borderPen.Dispose()
    $barBrush.Dispose()
    $textBrush.Dispose()
    $lockBrush.Dispose()
    $font.Dispose()
    $sf.Dispose()
    return $bmp
}

function Get-PngBytes {
    param([System.Drawing.Bitmap]$Bitmap)
    $ms = New-Object System.IO.MemoryStream
    $Bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $ms.ToArray()
    $ms.Dispose()
    return $bytes
}

# Generer les bitmaps pour chaque taille
$sizes = @(16, 32, 48, 256)
$pngSizes = [System.Collections.ArrayList]::new()
$pngDatas = [System.Collections.ArrayList]::new()
foreach ($sz in $sizes) {
    $bmp = New-ShieldBitmap -Size $sz
    $data = Get-PngBytes -Bitmap $bmp
    [void]$pngSizes.Add($sz)
    [void]$pngDatas.Add($data)
    $bmp.Dispose()
    Write-Host "  Size ${sz}x${sz}: $($data.Length) bytes PNG"
}

# Construire le fichier ICO
$imageCount = $pngSizes.Count
$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)

# ICONDIR header (6 octets)
$bw.Write([UInt16]0)              # Reserved
$bw.Write([UInt16]1)              # Type = 1 (icon)
$bw.Write([UInt16]$imageCount)    # Image count

# Calculer l'offset de debut des donnees image
$dataOffset = 6 + ($imageCount * 16)

# ICONDIRENTRY pour chaque image (16 octets chacune)
for ($i = 0; $i -lt $imageCount; $i++) {
    $sz = [int]$pngSizes[$i]
    $data = [byte[]]$pngDatas[$i]
    $widthByte = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }
    $heightByte = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }

    $bw.Write([byte]$widthByte)      # Width
    $bw.Write([byte]$heightByte)     # Height
    $bw.Write([byte]0)               # Color count
    $bw.Write([byte]0)               # Reserved
    $bw.Write([UInt16]1)             # Color planes
    $bw.Write([UInt16]32)            # Bits per pixel
    $bw.Write([UInt32]$data.Length)   # Image size
    $bw.Write([UInt32]$dataOffset)   # Offset

    $dataOffset += $data.Length
}

# Donnees PNG
for ($i = 0; $i -lt $imageCount; $i++) {
    $data = [byte[]]$pngDatas[$i]
    $bw.Write($data)
}

$bw.Flush()
$icoBytes = $ms.ToArray()
$bw.Dispose()
$ms.Dispose()

# Sauvegarder
$outPath = Join-Path $PSScriptRoot "app.ico"
[System.IO.File]::WriteAllBytes($outPath, $icoBytes)
Write-Host "Icon generated: $outPath ($($icoBytes.Length) bytes, $imageCount sizes)"
