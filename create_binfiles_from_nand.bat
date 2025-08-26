@echo off
setlocal

if not exist "nand.bin" (
  echo ERROR: nand.bin not found in %CD%
  exit /b 1
)

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$ErrorActionPreference='Stop';" ^
  "; function Copy-Bytes([string]$src,[long]$off,[int]$len,[string]$dst) {" ^
  "    $fs=[System.IO.File]::Open($src,'Open','Read');" ^
  "    try { $fs.Seek($off,'Begin') > $null; $buf=New-Object byte[] $len; $r=$fs.Read($buf,0,$len) } finally { $fs.Close() }" ^
  "    [System.IO.File]::WriteAllBytes($dst,$buf[0..($r-1)])" ^
  "}" ^
  "; Copy-Bytes 'nand.bin' 553648128 1024 'keys.bin'" ^
  "; Copy-Bytes 'keys.bin' 256 128 'otp.bin'" ^
  "; Copy-Bytes 'keys.bin' 512 256 'seeprom.bin'" ^
  "; $fs=[System.IO.File]::Open('keys.bin','Open','Read');" ^
  "   try { $buf=New-Object byte[] 256; $r=$fs.Read($buf,0,256) } finally { $fs.Close() }" ^
  "   $stdout=[Console]::OpenStandardOutput(); $stdout.Write($buf,0,$r); $stdout.Flush()"

endlocal
