@echo off
rem Runs the packaged launcher without changing system execution policy.
powershell.exe -NoProfile -File "%~dp0Play.ps1" %*
if errorlevel 1 pause
