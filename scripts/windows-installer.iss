; Script generated for Inno Setup 6
#define MyAppName "xtools"
#ifndef VERSION
  #define VERSION "0.2.4"
#endif
#define MyAppPublisher "xtools"
#define MyAppURL "https://github.com/lianchengwu/xtools"
#define MyAppExeName "xtools-host.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={{B58D2D49-3A31-4F2A-885E-379E69A7C12F}
AppName={#MyAppName}
AppVersion={#VERSION}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\target\dist
OutputBaseFilename=xtools-{#VERSION}-windows-x86_64-setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "开机自启动 xtools"; GroupDescription: "启动选项:"; Flags: unchecked

[Files]
Source: "..\target\release\xtools-host.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\xtools-time.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\xtools-json.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\xtools-trans.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion; DestName: "README.md"
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion; DestName: "LICENSE"

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: autostart

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
