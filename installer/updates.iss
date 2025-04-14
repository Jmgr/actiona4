const
  UpdateCheckNoUpdate = 0;
  UpdateCheckAvailable = 1;
  UpdateCheckError = 2;
  GetSettingFalse = 0;
  GetSettingTrue = 1;
  GetSettingError = 2;

function CheckForUpdateSetup(VersionString: PAnsiChar; VersionStringCapacity: Cardinal; DownloadUrl: PAnsiChar; DownloadUrlCapacity: Cardinal; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'check_for_update@files:inno_plugin.dll cdecl setuponly';

function GetUpdateCheckEnabledSetup(ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'get_update_check_enabled@files:inno_plugin.dll cdecl setuponly';

function SetUpdateCheckEnabledSetup(UpdateCheckEnabled: Integer; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'set_update_check_enabled@files:inno_plugin.dll cdecl setuponly';

function GetTelemetryEnabledSetup(ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'get_telemetry_enabled@files:inno_plugin.dll cdecl setuponly';

function SetTelemetryEnabledSetup(TelemetryEnabled: Integer; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'set_telemetry_enabled@files:inno_plugin.dll cdecl setuponly';

var
  UpdateOptionsPage: TWizardPage;
  CheckForNewVersionCheckbox: TNewCheckBox;
  EnableAutomaticUpdateChecksCheckbox: TNewCheckBox;
  EnableTelemetryCheckbox: TNewCheckBox;
  TelemetryDescriptionLabel: TNewStaticText;
  TelemetryEnableCommandLabel: TNewStaticText;
  TelemetryDisableCommandLabel: TNewStaticText;
  ExitForUpdateDownload: Boolean;

function TrimUpdateStringBuffer(const StringBuffer: AnsiString): String;
var
  NullIndex: Integer;
begin
  NullIndex := Pos(#0, StringBuffer);
  if NullIndex = 0 then
    Result := String(StringBuffer)
  else
    Result := String(Copy(StringBuffer, 1, NullIndex - 1));
end;

procedure ShowUpdateCheckFailureWarning(const ErrorMessage: String);
begin
  SuppressibleMsgBox(
    'Actiona could not check for a newer version before installing.' + #13#10 + #13#10 +
    'Setup will continue.' + #13#10 + #13#10 +
    'Error: ' + ErrorMessage,
    mbError,
    MB_OK,
    IDOK
  );
end;

function OpenUpdateDownloadPage(const DownloadUrl: String): Boolean;
var
  ErrorCode: Integer;
begin
  Result := ShellExecAsOriginalUser('open', DownloadUrl, '', '', SW_SHOWNORMAL, ewNoWait, ErrorCode);
  if Result then
    exit;

  SuppressibleMsgBox(
    'Actiona could not open the download page for the newer version.' + #13#10 + #13#10 +
    'Setup will continue.' + #13#10 + #13#10 +
    'Error code: ' + IntToStr(ErrorCode),
    mbError,
    MB_OK,
    IDOK
  );
end;

function ExecuteInstallerUpdateCheck(): Boolean;
var
  VersionString: AnsiString;
  DownloadUrl: AnsiString;
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
  LatestVersion: String;
  LatestDownloadUrl: String;
begin
  Result := True;

  if not CheckForNewVersionCheckbox.Checked then
    exit;

  SetLength(VersionString, 256);
  VersionString[1] := #0;
  SetLength(DownloadUrl, 2048);
  DownloadUrl[1] := #0;
  SetLength(ErrorBuffer, 2048);
  ErrorBuffer[1] := #0;

  ResultCode := CheckForUpdateSetup(
    PAnsiChar(VersionString),
    Length(VersionString),
    PAnsiChar(DownloadUrl),
    Length(DownloadUrl),
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );

  if ResultCode = UpdateCheckNoUpdate then
    exit;

  if ResultCode = UpdateCheckError then begin
    ShowUpdateCheckFailureWarning(TrimUpdateStringBuffer(ErrorBuffer));
    exit;
  end;

  if ResultCode <> UpdateCheckAvailable then begin
    ShowUpdateCheckFailureWarning('Unexpected update check result: ' + IntToStr(ResultCode));
    exit;
  end;

  LatestVersion := TrimUpdateStringBuffer(VersionString);
  LatestDownloadUrl := TrimUpdateStringBuffer(DownloadUrl);

  if LatestDownloadUrl = '' then begin
    ShowUpdateCheckFailureWarning('The update server returned an empty download URL.');
    exit;
  end;

  if SuppressibleMsgBox(
    'A newer version of Actiona Run (' + LatestVersion + ') is available.' + #13#10 + #13#10 +
    'Do you want to open the download page now and stop this installation?',
    mbConfirmation,
    MB_YESNO,
    IDNO
  ) <> IDYES then
    exit;

  if not OpenUpdateDownloadPage(LatestDownloadUrl) then
    exit;

  ExitForUpdateDownload := True;
  WizardForm.Close;
  Result := False;
end;

procedure SaveUpdateCheckSetting();
var
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
  UpdateCheckEnabled: Integer;
begin
  UpdateCheckEnabled := 0;
  if EnableAutomaticUpdateChecksCheckbox.Checked then
    UpdateCheckEnabled := 1;

  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;
  ResultCode := SetUpdateCheckEnabledSetup(
    UpdateCheckEnabled,
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );

  if ResultCode = 0 then
    exit;

  SuppressibleMsgBox(
    'Actiona could not save the automatic update setting.' + #13#10 + #13#10 +
    'You may need to update it later from the application.' + #13#10 + #13#10 +
    'Error: ' + TrimUpdateStringBuffer(ErrorBuffer),
    mbError,
    MB_OK,
    IDOK
  );
end;

procedure SaveTelemetrySetting();
var
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
  TelemetryEnabled: Integer;
begin
  TelemetryEnabled := 0;
  if EnableTelemetryCheckbox.Checked then
    TelemetryEnabled := 1;

  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;
  ResultCode := SetTelemetryEnabledSetup(
    TelemetryEnabled,
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );

  if ResultCode = 0 then
    exit;

  SuppressibleMsgBox(
    'Actiona could not save the telemetry setting.' + #13#10 + #13#10 +
    'You may need to update it later from the application.' + #13#10 + #13#10 +
    'Error: ' + TrimUpdateStringBuffer(ErrorBuffer),
    mbError,
    MB_OK,
    IDOK
  );
end;

procedure ShowSettingsLoadFailureWarning(const SettingName, ErrorMessage: String);
begin
  SuppressibleMsgBox(
    'Actiona could not read the saved ' + SettingName + ' setting.' + #13#10 + #13#10 +
    'Setup will use the default value for this install.' + #13#10 + #13#10 +
    'Error: ' + ErrorMessage,
    mbError,
    MB_OK,
    IDOK
  );
end;

function LoadSavedUpdateCheckSettingOrDefault(const SettingName: String; const DefaultValue: Boolean): Boolean;
var
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
begin
  Result := DefaultValue;

  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;
  ResultCode := GetUpdateCheckEnabledSetup(PAnsiChar(ErrorBuffer), Length(ErrorBuffer));

  if ResultCode = GetSettingFalse then begin
    Result := False;
    exit;
  end;

  if ResultCode = GetSettingTrue then begin
    Result := True;
    exit;
  end;

  if ResultCode = GetSettingError then begin
    ShowSettingsLoadFailureWarning(SettingName, TrimUpdateStringBuffer(ErrorBuffer));
    exit;
  end;

  ShowSettingsLoadFailureWarning(
    SettingName,
    'Unexpected setting load result: ' + IntToStr(ResultCode)
  );
end;

function LoadSavedTelemetrySettingOrDefault(const SettingName: String; const DefaultValue: Boolean): Boolean;
var
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
begin
  Result := DefaultValue;

  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;
  ResultCode := GetTelemetryEnabledSetup(PAnsiChar(ErrorBuffer), Length(ErrorBuffer));

  if ResultCode = GetSettingFalse then begin
    Result := False;
    exit;
  end;

  if ResultCode = GetSettingTrue then begin
    Result := True;
    exit;
  end;

  if ResultCode = GetSettingError then begin
    ShowSettingsLoadFailureWarning(SettingName, TrimUpdateStringBuffer(ErrorBuffer));
    exit;
  end;

  ShowSettingsLoadFailureWarning(
    SettingName,
    'Unexpected setting load result: ' + IntToStr(ResultCode)
  );
end;

procedure InitializeWizard();
begin
  UpdateOptionsPage := CreateCustomPage(
    wpSelectTasks,
    'Updates and telemetry',
    'Choose how setup should handle version checks and telemetry for Actiona.'
  );

  CheckForNewVersionCheckbox := TNewCheckBox.Create(UpdateOptionsPage.Surface);
  CheckForNewVersionCheckbox.Parent := UpdateOptionsPage.Surface;
  CheckForNewVersionCheckbox.Left := 0;
  CheckForNewVersionCheckbox.Top := 0;
  CheckForNewVersionCheckbox.Width := UpdateOptionsPage.SurfaceWidth;
  CheckForNewVersionCheckbox.Height := ScaleY(17);
  CheckForNewVersionCheckbox.Checked := LoadSavedUpdateCheckSettingOrDefault(
    'automatic update check',
    True
  );
  CheckForNewVersionCheckbox.Caption := 'Check for a newer version before installing';

  EnableAutomaticUpdateChecksCheckbox := TNewCheckBox.Create(UpdateOptionsPage.Surface);
  EnableAutomaticUpdateChecksCheckbox.Parent := UpdateOptionsPage.Surface;
  EnableAutomaticUpdateChecksCheckbox.Left := 0;
  EnableAutomaticUpdateChecksCheckbox.Top := CheckForNewVersionCheckbox.Top + CheckForNewVersionCheckbox.Height + ScaleY(8);
  EnableAutomaticUpdateChecksCheckbox.Width := UpdateOptionsPage.SurfaceWidth;
  EnableAutomaticUpdateChecksCheckbox.Height := ScaleY(17);
  EnableAutomaticUpdateChecksCheckbox.Checked := LoadSavedUpdateCheckSettingOrDefault(
    'automatic update checks',
    True
  );
  EnableAutomaticUpdateChecksCheckbox.Caption := 'Enable automatic update checks';

  EnableTelemetryCheckbox := TNewCheckBox.Create(UpdateOptionsPage.Surface);
  EnableTelemetryCheckbox.Parent := UpdateOptionsPage.Surface;
  EnableTelemetryCheckbox.Left := 0;
  EnableTelemetryCheckbox.Top := EnableAutomaticUpdateChecksCheckbox.Top + EnableAutomaticUpdateChecksCheckbox.Height + ScaleY(8);
  EnableTelemetryCheckbox.Width := UpdateOptionsPage.SurfaceWidth;
  EnableTelemetryCheckbox.Height := ScaleY(17);
  EnableTelemetryCheckbox.Checked := LoadSavedTelemetrySettingOrDefault(
    'telemetry',
    False
  );
  EnableTelemetryCheckbox.Caption := 'Help us improve Actiona (anonymous usage data)';

  TelemetryDescriptionLabel := TNewStaticText.Create(UpdateOptionsPage.Surface);
  TelemetryDescriptionLabel.Parent := UpdateOptionsPage.Surface;
  TelemetryDescriptionLabel.Left := ScaleX(20);
  TelemetryDescriptionLabel.Top := EnableTelemetryCheckbox.Top + EnableTelemetryCheckbox.Height + ScaleY(4);
  TelemetryDescriptionLabel.Width := UpdateOptionsPage.SurfaceWidth - ScaleX(20);
  TelemetryDescriptionLabel.Height := ScaleY(120);
  TelemetryDescriptionLabel.AutoSize := False;
  TelemetryDescriptionLabel.WordWrap := True;
  TelemetryDescriptionLabel.Caption :=
    'This helps us understand which features are most used and prioritize improvements.' + #13#10 + #13#10 +
    'What we collect (pseudonymized):' + #13#10 +
    '  - Screen resolution' + #13#10 +
    '  - OS name, version, architecture, and locale' + #13#10 +
    '  - CPU, GPU, and RAM information' + #13#10 +
    '  - Script running time, API calls, file extensions, memory usage, and actions used' + #13#10 + #13#10 +
    'You can change this anytime from the command line:';

  TelemetryEnableCommandLabel := TNewStaticText.Create(UpdateOptionsPage.Surface);
  TelemetryEnableCommandLabel.Parent := UpdateOptionsPage.Surface;
  TelemetryEnableCommandLabel.Left := ScaleX(20);
  TelemetryEnableCommandLabel.Top := TelemetryDescriptionLabel.Top + TelemetryDescriptionLabel.Height + ScaleY(4);
  TelemetryEnableCommandLabel.Width := UpdateOptionsPage.SurfaceWidth - ScaleX(20);
  TelemetryEnableCommandLabel.Height := ScaleY(17);
  TelemetryEnableCommandLabel.AutoSize := False;
  TelemetryEnableCommandLabel.Font.Name := 'Consolas';
  TelemetryEnableCommandLabel.Font.Style := [fsBold];
  TelemetryEnableCommandLabel.Caption := 'Enable:  actiona-run config telemetry true';

  TelemetryDisableCommandLabel := TNewStaticText.Create(UpdateOptionsPage.Surface);
  TelemetryDisableCommandLabel.Parent := UpdateOptionsPage.Surface;
  TelemetryDisableCommandLabel.Left := ScaleX(20);
  TelemetryDisableCommandLabel.Top := TelemetryEnableCommandLabel.Top + TelemetryEnableCommandLabel.Height + ScaleY(2);
  TelemetryDisableCommandLabel.Width := UpdateOptionsPage.SurfaceWidth - ScaleX(20);
  TelemetryDisableCommandLabel.Height := ScaleY(17);
  TelemetryDisableCommandLabel.AutoSize := False;
  TelemetryDisableCommandLabel.Font.Name := 'Consolas';
  TelemetryDisableCommandLabel.Font.Style := [fsBold];
  TelemetryDisableCommandLabel.Caption := 'Disable: actiona-run config telemetry false';
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;

  if CurPageID <> UpdateOptionsPage.ID then
    exit;

  Result := ExecuteInstallerUpdateCheck();
end;

procedure CancelButtonClick(CurPageID: Integer; var Cancel, Confirm: Boolean);
begin
  if ExitForUpdateDownload then
    Confirm := False;
end;
