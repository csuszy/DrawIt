@echo off
title DrawIt - Fejlesztoi Tanusitvany Telepito
echo ========================================================
echo   DrawIt Fejlesztoi Tanusitvany Telepito (LocalMachine)
echo ========================================================
echo.
echo Rendszergazdai engedely kerese a tanusitvany rogzitesehez...
powershell -Command "Start-Process powershell -Verb RunAs -ArgumentList '-NoExit', '-Command', 'Import-Certificate -FilePath \"\"%~dp0DrawIt_Dev.cer\"\" -CertStoreLocation Cert:\LocalMachine\TrustedPeople; Write-Host \"`n========================================================`nSikeresen telepitve a Megbizhato szemelyek (TrustedPeople) koze!`nMost mar ketszer rakattintva siman telepul a DrawIt.msix!`n========================================================\" -ForegroundColor Green'"
