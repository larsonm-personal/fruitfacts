@echo off
setlocal

rem adds files to dvc (data version control, for large files that are too big for git)

for /f "usebackq delims=" %%I in (`powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0helper_scripts\dependency_env.ps1" -Name dvc -RepoRoot "%~dp0." -EmitBatchEnv`) do %%I
if not defined DVC_EXE goto :missing_dvc

echo Using DVC: %DVC_EXE%
if /I "%~1"=="--resolve-only" goto :EOF
if /I "%~1"=="--commands-only" goto :commands_only

call "%DVC_EXE%" add --glob "plant_database\**\*.pdf" || goto :error
call "%DVC_EXE%" add --glob "frontend\public\data\**\*.jpg" || goto :error
call "%DVC_EXE%" diff || goto :error
call "%DVC_EXE%" push || goto :error

echo next step is to add .dvc files to git

goto :EOF

:commands_only
echo call "%DVC_EXE%" add --glob "plant_database\**\*.pdf"
echo call "%DVC_EXE%" add --glob "frontend\public\data\**\*.jpg"
echo call "%DVC_EXE%" diff
echo call "%DVC_EXE%" push
goto :EOF

:missing_dvc
echo Failed to find DVC executable
echo Run helper_scripts\dvc_install_or_update.ps1 if DVC is not installed
exit /b 1

:error
echo Failed with error #%errorlevel%.
exit /b %errorlevel%
