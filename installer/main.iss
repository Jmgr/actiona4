#define MyAppName "Actiona Run"
#ifndef MyAppVersion
  #error Missing MyAppVersion define. Pass /DMyAppVersion=... to ISCC.
#endif
#ifndef MyAppFileVersion
  #error Missing MyAppFileVersion define. Pass /DMyAppFileVersion=... to ISCC.
#endif
#ifndef MySignTool
  #error Missing MySignTool define. Pass /DMySignTool=... to ISCC.
#endif
#ifndef MyAppPublisher
  #error Missing MyAppPublisher define. Pass /DMyAppPublisher=... to ISCC.
#endif
#ifndef MyAppURL
  #error Missing MyAppURL define. Pass /DMyAppURL=... to ISCC.
#endif
#ifndef MyNotificationAUMID
  #error Missing MyNotificationAUMID define. Pass /DMyNotificationAUMID=... to ISCC.
#endif
#ifndef MyNotificationDisplayName
  #error Missing MyNotificationDisplayName define. Pass /DMyNotificationDisplayName=... to ISCC.
#endif
#define MyAppExeName "actiona-runw.exe"

[Setup]
AppId={{A3D5D4F0-1AFA-4278-9E23-FA4A36632447}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
SetupIconFile=..\crates\core\icons\icon.ico
VersionInfoVersion={#MyAppFileVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} Setup
VersionInfoTextVersion={#MyAppVersion}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DisableProgramGroupPage=yes
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\target
OutputBaseFilename=actiona-run-{#MyAppVersion}
#if MySignTool != ""
  SignTool={#MySignTool}
  SignedUninstaller=yes
#endif
ChangesEnvironment=yes
SolidCompression=yes
WizardStyle=modern
WizardSmallImageFile=..\crates\core\icons\icon.png

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "addtopath"; Description: "Add application directory to PATH"

[Files]
#include "..\target\files.iss"

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[Code]
#include "path.iss"
#include "notification.iss"
#include "updates.iss"

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep <> ssPostInstall then
    exit;

  SaveUpdateCheckSetting();
  SaveTelemetrySetting();
  ExecuteSetupNotificationRegistration();

  if WizardIsTaskSelected('addtopath') then
    ExecuteSetupPathOperation(True, ExpandConstant('{app}'))
  else
    ExecuteSetupPathOperation(False, ExpandConstant('{app}'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then begin
    ExecuteUninstallNotificationUnregistration();
    ExecuteUninstallPathOperation(ExpandConstant('{app}'));
  end;
end;
