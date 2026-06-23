@echo off
setlocal

set "BASH_EXE="

for %%P in (
  "%ProgramFiles%\Git\bin\bash.exe"
  "%ProgramFiles%\Git\usr\bin\bash.exe"
  "%ProgramFiles(x86)%\Git\bin\bash.exe"
  "%ProgramFiles(x86)%\Git\usr\bin\bash.exe"
  "%LocalAppData%\Programs\Git\bin\bash.exe"
  "%LocalAppData%\Programs\Git\usr\bin\bash.exe"
  "C:\msys64\usr\bin\bash.exe"
  "C:\tools\msys64\usr\bin\bash.exe"
) do (
  if exist "%%~P" (
    set "BASH_EXE=%%~P"
    goto :run
  )
)

for /f "delims=" %%P in ('where bash 2^>nul') do (
  set "BASH_EXE=%%P"
  goto :run
)

echo Git Bash was not found. Install Git for Windows or add bash.exe to PATH on the windows-x64 Buildkite runner. 1>&2
exit /b 127

:run
"%BASH_EXE%" %*
exit /b %ERRORLEVEL%
