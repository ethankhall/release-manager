FROM microsoft/windowsservercore

RUN powershell -Command \
  Set-ExecutionPolicy Bypass -Scope Process -Force; \
  iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

RUN choco install --yes rust