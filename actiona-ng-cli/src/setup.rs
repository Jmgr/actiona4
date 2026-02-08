/// Performs automatic platform-specific setup that should run on every invocation.
/// Currently this only does work on Windows (notification app registration).
pub fn ensure_platform_setup() {
    #[cfg(windows)]
    ensure_notification_registration();
}

#[cfg(windows)]
fn ensure_notification_registration() {
    use std::{fs, path::PathBuf};

    use windows::{
        Win32::{
            System::Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, IPersistFile,
            },
            UI::Shell::{
                IShellLinkW,
                PropertiesSystem::{
                    GPS_READWRITE, IPropertyStore, PROPERTYKEY, PSGetPropertyKeyFromName,
                    SHGetPropertyStoreFromParsingName,
                },
                ShellLink,
            },
        },
        core::{HSTRING, Interface},
    };

    const AUMID: &str = "Actiona.ActionaNg";

    let result = (|| -> color_eyre::Result<()> {
        use color_eyre::eyre::Context;

        let exe_path = std::env::current_exe().context("getting executable path")?;
        let exe_str = exe_path
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("executable path is not valid UTF-8"))?;

        // Get Start Menu Programs path
        let start_menu = {
            let appdata = std::env::var("APPDATA").context("APPDATA not set")?;
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
        };

        let shortcut_path = start_menu.join("Actiona.lnk");

        // Quick check: if shortcut exists and points to the current executable, skip
        if shortcut_path.exists() {
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

                let shell_link: IShellLinkW =
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

                let persist_file: IPersistFile = Interface::cast(&shell_link)?;

                let shortcut_hstring = HSTRING::from(shortcut_path.to_str().unwrap_or_default());

                if persist_file.Load(&shortcut_hstring, 0).is_ok() {
                    let mut target_buf = [0u16; 260];
                    if shell_link
                        .GetPath(&mut target_buf, std::ptr::null_mut(), 0)
                        .is_ok()
                    {
                        let target = String::from_utf16_lossy(&target_buf);
                        let target = target.trim_end_matches('\0');
                        if target.eq_ignore_ascii_case(exe_str) {
                            CoUninitialize();
                            return Ok(()); // Already registered with correct path
                        }
                    }
                }

                CoUninitialize();
            }
        }

        // Create or update the shortcut
        fs::create_dir_all(&start_menu).context("creating Start Menu directory")?;

        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).context("COM initialization")?;

            let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .context("creating ShellLink")?;

            shell_link
                .SetPath(&HSTRING::from(exe_str))
                .context("setting shortcut path")?;

            shell_link
                .SetDescription(&HSTRING::from("Actiona - Desktop Automation"))
                .context("setting shortcut description")?;

            let persist_file: IPersistFile =
                Interface::cast(&shell_link).context("casting to IPersistFile")?;

            let shortcut_hstring = HSTRING::from(
                shortcut_path
                    .to_str()
                    .ok_or_else(|| color_eyre::eyre::eyre!("shortcut path not valid UTF-8"))?,
            );

            persist_file
                .Save(&shortcut_hstring, true)
                .context("saving shortcut")?;

            // Set the AUMID property on the shortcut
            let mut store: Option<IPropertyStore> = None;
            SHGetPropertyStoreFromParsingName(&shortcut_hstring, None, GPS_READWRITE, &mut store)
                .context("getting property store")?;

            if let Some(store) = store {
                use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;

                let pk_aumid = {
                    let mut pk = PROPERTYKEY::default();
                    PSGetPropertyKeyFromName(&HSTRING::from("System.AppUserModel.ID"), &mut pk)
                        .context("getting AUMID property key")?;
                    pk
                };

                let aumid_variant = PROPVARIANT::from(HSTRING::from(AUMID));
                store
                    .SetValue(&pk_aumid, &aumid_variant)
                    .context("setting AUMID")?;
                store.Commit().context("committing property store")?;
            }

            CoUninitialize();
        }

        Ok(())
    })();

    if let Err(e) = result {
        // Silent failure — don't bother the user, notifications just won't work
        tracing::warn!("Could not register notification app: {e}");
    }
}
