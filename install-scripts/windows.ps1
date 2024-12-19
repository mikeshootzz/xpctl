# Set the download URL
$RepoURL = "https://github.com/mikeshootzz/xpctl/releases/download/v1.0.0-alpha.3"
$Filename = "xpctl-x86_64-pc-windows-msvc.zip"
$DownloadURL = "$RepoURL/$Filename"
$TempPath = "$env:TEMP\$Filename"

# Download the file
Write-Host "Downloading $DownloadURL..."
Invoke-WebRequest -Uri $DownloadURL -OutFile $TempPath

# Extract the ZIP file
$ExtractPath = "$env:TEMP\xpctl"
Expand-Archive -Path $TempPath -DestinationPath $ExtractPath -Force

# Move the binary to C:\Windows\System32 (requires admin rights)
Move-Item -Path "$ExtractPath\xpctl.exe" -Destination "C:\Windows\System32" -Force

# Clean up
Remove-Item -Path $TempPath -Force
Remove-Item -Path $ExtractPath -Recurse -Force

Write-Host "Installation complete!"
Write-Host "Please set the XPCTL_API_KEY environment variable to your API key of XPipe"
