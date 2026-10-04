use std::{process, sync::Arc};

use color_eyre::Result;
use itertools::Itertools;
use tracing::{info, warn};
use x11rb_async::{
    connection::Connection,
    errors::ReplyError,
    protocol::{
        ErrorKind,
        xproto::{
            Atom, AtomEnum, ConnectionExt, GrabMode, KeyPressEvent, Keycode, ModMask, PropMode,
            Property, PropertyNotifyEvent,
        },
    },
};
use xkbcommon::xkb::{self, Keysym};

use crate::{platform::x11::X11Connection, runtime::hotkey::Hotkey};

/// Lock-style modifiers that must not prevent the hotkey from firing: Caps Lock and Num Lock
/// (`Mod2` on virtually every X11 keymap). A passive grab only matches the exact modifier state,
/// so the hotkey is grabbed once for every combination of these.
fn ignored_modifiers() -> [ModMask; 4] {
    [
        ModMask::from(0_u16),
        ModMask::LOCK,
        ModMask::M2,
        ModMask::LOCK | ModMask::M2,
    ]
}

/// Where the hotkey is grabbed, resolved from the keyboard mapping when grabbing starts.
struct GrabTarget {
    keycode: Keycode,
    /// Modifier states that trigger the hotkey, Lock and Num Lock aside.
    modifier_states: Vec<ModMask>,
    /// Which of `grabs()` this instance holds, in the same order.
    grabbed: Vec<bool>,
}

impl GrabTarget {
    fn new(keycode: Keycode, modifier_states: Vec<ModMask>) -> Self {
        let grab_count = modifier_states.len() * ignored_modifiers().len();
        Self {
            keycode,
            modifier_states,
            grabbed: vec![false; grab_count],
        }
    }

    /// Every modifier mask to grab: each modifier state combined with each lock variant.
    fn grabs(&self) -> impl Iterator<Item = ModMask> + use<'_> {
        self.modifier_states.iter().flat_map(|&modifier_state| {
            ignored_modifiers()
                .into_iter()
                .map(move |ignored| modifier_state | ignored)
        })
    }

    fn is_complete(&self) -> bool {
        self.grabbed.iter().all(|&grabbed| grabbed)
    }
}

/// Global hotkey that stops every running instance sharing it.
///
/// Only one X11 client can grab a given key combination, so the first instance owns the grab
/// and the others retry until it is released. When the owner sees the hotkey, it changes a root
/// window property named after the hotkey; every instance (owner included) watches that property
/// and stops when it changes.
///
/// A hotkey is several grabs: one per modifier state (see `modifier_states`) and lock-modifier
/// variant. Each is grabbed and kept independently, and only the missing ones are retried. The
/// grabs are separate requests, so instances grabbing at the same time can end up splitting them.
/// That is harmless since any of them broadcasts the stop, whereas releasing partial grabs on
/// failure could leave nobody holding the hotkey while they keep colliding.
///
/// Grabbing a single combination means the server only reports that combination to us: no other
/// key press is seen.
pub struct StopHotkey {
    x11_connection: Arc<X11Connection>,
    hotkey: Hotkey,
    keysym: Keysym,
    modifiers: ModMask,
    stop_atom: Atom,
    target: Option<GrabTarget>,
    reported_failure: bool,
}

impl StopHotkey {
    pub async fn new(x11_connection: Arc<X11Connection>, hotkey: Hotkey) -> Result<Self> {
        let connection = x11_connection.async_connection();
        let atom_name = format!("_ACTIONA_RUN_STOP {hotkey}");
        let stop_atom = connection
            .intern_atom(false, atom_name.as_bytes())
            .await?
            .reply()
            .await?
            .atom;

        let mut modifiers = ModMask::from(0_u16);
        for (enabled, modifier) in [
            (hotkey.ctrl, ModMask::CONTROL),
            (hotkey.alt, ModMask::M1),
            (hotkey.shift, ModMask::SHIFT),
            (hotkey.meta, ModMask::M4),
        ] {
            if enabled {
                modifiers |= modifier;
            }
        }

        Ok(Self {
            x11_connection,
            keysym: hotkey.key.into(),
            hotkey,
            modifiers,
            stop_atom,
            target: None,
            reported_failure: false,
        })
    }

    /// Whether this instance holds every grab, so there is nothing left to retry.
    pub fn is_grabbed(&self) -> bool {
        self.target.as_ref().is_some_and(GrabTarget::is_complete)
    }

    /// Tries to grab what this instance does not hold yet. Failing because another client holds
    /// some of it is not an error: the caller retries later.
    pub async fn try_grab(&mut self, keymap: &xkb::Keymap) -> Result<()> {
        if self.target.is_none() {
            self.target = self.find_target(keymap).await?;
        }
        let Some(target) = &mut self.target else {
            if !self.reported_failure {
                warn!(
                    "stop hotkey {}: the key is not on the current keyboard layout",
                    self.hotkey
                );
                self.reported_failure = true;
            }
            return Ok(());
        };
        if target.is_complete() {
            return Ok(());
        }

        let connection = self.x11_connection.async_connection();
        let root = self.x11_connection.screen().root;
        let mut cookies = Vec::with_capacity(target.grabbed.len());
        for (index, modifiers) in target.grabs().enumerate() {
            if target.grabbed[index] {
                continue;
            }

            let cookie = connection
                .grab_key(
                    false,
                    root,
                    modifiers,
                    target.keycode,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .await?;
            cookies.push((index, cookie));
        }

        let mut is_busy = false;
        let mut first_error = None;
        for (index, cookie) in cookies {
            match cookie.check().await {
                Ok(()) => target.grabbed[index] = true,
                Err(ReplyError::X11Error(error)) if error.error_kind == ErrorKind::Access => {
                    is_busy = true;
                }
                Err(err) => {
                    first_error.get_or_insert(err);
                }
            }
        }

        if target.is_complete() {
            info!("stop hotkey {} grabbed", self.hotkey);
            self.reported_failure = false;
        } else if is_busy && !self.reported_failure {
            // Usually another actiona-run instance, which stops us through the root window
            // property anyway; it can also be an unrelated application.
            info!(
                "stop hotkey {} is held by another client, retrying",
                self.hotkey
            );
            self.reported_failure = true;
        }

        first_error.map_or(Ok(()), |err| Err(err.into()))
    }

    /// Releases every grab this instance holds, e.g. because the keyboard mapping changed and the
    /// key or AltGr may now be elsewhere.
    pub async fn ungrab(&mut self) -> Result<()> {
        let Some(target) = self.target.take() else {
            return Ok(());
        };

        let connection = self.x11_connection.async_connection();
        let root = self.x11_connection.screen().root;
        for (modifiers, grabbed) in target.grabs().zip(&target.grabbed) {
            if *grabbed {
                connection
                    .ungrab_key(target.keycode, root, modifiers)
                    .await?;
            }
        }
        connection.flush().await?;

        Ok(())
    }

    pub fn is_hotkey_press(&self, event: &KeyPressEvent) -> bool {
        let Some(target) = &self.target else {
            return false;
        };

        let relevant = target.modifier_states.iter().fold(
            ModMask::CONTROL | ModMask::M1 | ModMask::SHIFT | ModMask::M4,
            |relevant, &modifier_state| relevant | modifier_state,
        );
        let state = u16::from(event.state) & u16::from(relevant);

        target.keycode == event.detail
            && event.root == self.x11_connection.screen().root
            && target
                .modifier_states
                .iter()
                .any(|&modifier_state| u16::from(modifier_state) == state)
    }

    /// Asks every instance sharing this hotkey to stop, this one included.
    pub async fn broadcast_stop(&self) -> Result<()> {
        let connection = self.x11_connection.async_connection();
        let value = process::id().to_ne_bytes();
        connection
            .change_property(
                PropMode::REPLACE,
                self.x11_connection.screen().root,
                self.stop_atom,
                AtomEnum::CARDINAL,
                32,
                1,
                &value,
            )
            .await?;
        connection.flush().await?;

        Ok(())
    }

    pub fn is_stop_broadcast(&self, event: &PropertyNotifyEvent) -> bool {
        event.window == self.x11_connection.screen().root
            && event.atom == self.stop_atom
            && event.state == Property::NEW_VALUE
    }

    async fn find_target(&self, keymap: &xkb::Keymap) -> Result<Option<GrabTarget>> {
        let Some(keycode) = keycodes_with_keysym(keymap, self.keysym).next() else {
            return Ok(None);
        };

        let mut modifier_states = vec![self.modifiers];

        // Windows reports AltGr as Ctrl+Alt, so a Ctrl+Alt hotkey also fires with AltGr there.
        // On X11 AltGr is a modifier of its own (usually Mod5): accept it in place of Ctrl+Alt,
        // with or without Ctrl, to behave the same.
        if self.hotkey.ctrl
            && self.hotkey.alt
            && let Some(altgr) = self.altgr_modifier(keymap).await?
        {
            let others = self.modifiers.remove(ModMask::CONTROL | ModMask::M1);
            for modifier_state in [others | altgr, others | altgr | ModMask::CONTROL] {
                if !modifier_states.contains(&modifier_state) {
                    modifier_states.push(modifier_state);
                }
            }
        }

        Ok(Some(GrabTarget::new(keycode, modifier_states)))
    }

    /// The modifier AltGr (`ISO_Level3_Shift`) sets, if the layout has one.
    async fn altgr_modifier(&self, keymap: &xkb::Keymap) -> Result<Option<ModMask>> {
        let altgr_keycodes = keycodes_with_keysym(keymap, Keysym::ISO_Level3_Shift).collect_vec();
        if altgr_keycodes.is_empty() {
            return Ok(None);
        }

        let mapping = self
            .x11_connection
            .async_connection()
            .get_modifier_mapping()
            .await?
            .reply()
            .await?;
        let keycodes_per_modifier = usize::from(mapping.keycodes_per_modifier()).max(1);

        // Modifiers are listed in order: Shift, Lock, Control, Mod1 to Mod5.
        Ok(mapping
            .keycodes
            .chunks(keycodes_per_modifier)
            .position(|keycodes| {
                keycodes
                    .iter()
                    .any(|keycode| altgr_keycodes.contains(keycode))
            })
            .and_then(|index| u16::try_from(index).ok())
            .map(|index| ModMask::from(1_u16 << index)))
    }
}

/// Keycodes producing `keysym` at any shift level of the first layout.
fn keycodes_with_keysym(keymap: &xkb::Keymap, keysym: Keysym) -> impl Iterator<Item = Keycode> {
    let min = keymap.min_keycode().raw();
    let max = keymap.max_keycode().raw();
    (min..=max).filter_map(move |raw| {
        let keycode = xkb::Keycode::new(raw);
        let levels = keymap.num_levels_for_key(keycode, 0);
        (0..levels)
            .any(|level| {
                keymap
                    .key_get_syms_by_level(keycode, 0, level)
                    .contains(&keysym)
            })
            .then(|| Keycode::try_from(raw).ok())
            .flatten()
    })
}
