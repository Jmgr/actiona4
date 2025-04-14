function RegisterNotificationAppSetup(Aumid: PAnsiChar; DisplayName: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'register_notification_app@files:inno_plugin.dll cdecl setuponly';

function UnregisterNotificationAppUninstall(Aumid: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'unregister_notification_app@{app}\inno_plugin.dll cdecl uninstallonly';

function TrimNotificationErrorBuffer(const ErrorBuffer: AnsiString): AnsiString;
var
  NullIndex: Integer;
begin
  NullIndex := Pos(#0, ErrorBuffer);
  if NullIndex = 0 then
    Result := ErrorBuffer
  else
    Result := Copy(ErrorBuffer, 1, NullIndex - 1);
end;

procedure HandleNotificationOperationResult(const ResultCode: Integer; const OperationName: String; const ErrorBuffer: AnsiString; const RaiseOnError: Boolean);
var
  ErrorMessage: AnsiString;
  PromptMessage: String;
begin
  if ResultCode = 0 then
    exit;

  ErrorMessage := TrimNotificationErrorBuffer(ErrorBuffer);
  if ErrorMessage = '' then
    ErrorMessage := 'Unknown error.';

  if RaiseOnError then
    RaiseException(Format('Failed to %s notification registration: %s', [OperationName, String(ErrorMessage)]))
  else begin
    Log(Format('Failed to %s notification registration during uninstall: %s', [OperationName, String(ErrorMessage)]));
    PromptMessage := Format('Actiona could not %s its notification registration during uninstall.' + #13#10 + #13#10 + 'You may need to clean it up manually.' + #13#10 + #13#10 + 'Error: %s', [OperationName, String(ErrorMessage)]);
    SuppressibleMsgBox(
      PromptMessage,
      mbError,
      MB_OK,
      IDOK
    );
  end;
end;

procedure ExecuteSetupNotificationRegistration();
var
  AumidAnsi: AnsiString;
  DisplayNameAnsi: AnsiString;
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
begin
  AumidAnsi := '{#MyNotificationAUMID}';
  DisplayNameAnsi := '{#MyNotificationDisplayName}';
  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;

  ResultCode := RegisterNotificationAppSetup(
    PAnsiChar(AumidAnsi),
    PAnsiChar(DisplayNameAnsi),
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );

  HandleNotificationOperationResult(ResultCode, 'register', ErrorBuffer, True);
end;

procedure ExecuteUninstallNotificationUnregistration();
var
  AumidAnsi: AnsiString;
  ErrorBuffer: AnsiString;
  PluginDllPath: String;
  ResultCode: Integer;
begin
  AumidAnsi := '{#MyNotificationAUMID}';
  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;
  PluginDllPath := ExpandConstant('{app}\inno_plugin.dll');

  ResultCode := UnregisterNotificationAppUninstall(
    PAnsiChar(AumidAnsi),
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );
  UnloadDLL(PluginDllPath);

  HandleNotificationOperationResult(ResultCode, 'unregister', ErrorBuffer, False);
end;
