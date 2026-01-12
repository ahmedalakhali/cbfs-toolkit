@echo off
REM ============================================================================
REM CBFS Toolkit - Release Build & Packaging Script (Batch Wrapper)
REM ============================================================================
REM Double-click this file to build and package the release.
REM ============================================================================

echo.
echo ========================================
echo   CBFS Toolkit - Release Builder
echo ========================================
echo.

REM Run the PowerShell script
powershell -ExecutionPolicy Bypass -File "%~dp0build-release.ps1"

echo.
pause
