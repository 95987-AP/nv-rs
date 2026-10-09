@echo off
setlocal DisableDelayedExpansion
set "nvrs_viewer=%NV_RS_VIEWER%"
if not defined nvrs_viewer set "nvrs_viewer=%~dp0app\nv-viewer.exe"
rem Interactive launcher needs no PowerShell execution-policy change.
set "nvrs_data=%NV_RS_DATA%"
if not defined nvrs_data set /p "nvrs_data=Fallout New Vegas Data folder: "
if not defined nvrs_data exit /b 1
set "nvrs_data=%nvrs_data:"=%"
if not exist "%nvrs_data%\FalloutNV.esm" (
  echo FalloutNV.esm was not found in that folder.
  pause
  exit /b 1
)
if not exist "%nvrs_viewer%" (
  echo Extract the entire playtest ZIP before launching.
  pause
  exit /b 1
)
echo 1. Explore Doc Mitchell's house
echo 2. Start the experimental opening
echo 3. Explore Goodsprings
choice /c 123 /n /m "Choose 1, 2 or 3: "
set "nvrs_choice=%errorlevel%"
echo 1. Diagnostics off
echo 2. Diagnostics on
choice /c 12 /n /m "Record performance and loading evidence? Choose 1 or 2: "
set "nvrs_diagnostics_choice=%errorlevel%"
set "nvrs_diagnostics="
if "%nvrs_diagnostics_choice%"=="2" set "nvrs_diagnostics=--diagnostics"
if not exist "%~dp0userdata" mkdir "%~dp0userdata"
cd /d "%~dp0userdata"
if defined NV_RS_VIEWER (
  echo Local development build: "%nvrs_viewer%"
) else (
  type "%~dp0BUILD.txt"
)
if "%nvrs_choice%"=="3" "%nvrs_viewer%" "%nvrs_data%" Goodsprings --official %nvrs_diagnostics%
if "%nvrs_choice%"=="2" "%nvrs_viewer%" "%nvrs_data%" --new-game --official %nvrs_diagnostics%
if "%nvrs_choice%"=="1" "%nvrs_viewer%" "%nvrs_data%" GSDocMitchellHouse --official %nvrs_diagnostics%
if errorlevel 1 pause
