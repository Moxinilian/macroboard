use std::{
    collections::{HashMap, HashSet},
    ops::{BitOr, BitOrAssign},
};

use keycode::KeyMappingCode;
use tinyset::SetU32;

struct Trigger {
    trigger: SetU32,
    on_pressed: Box<dyn FnMut() -> ListeningCmd>,
    on_released: Option<Box<dyn FnMut() -> ListeningCmd>>,
}

/// Commands the listening server to stop or
/// continue.
#[derive(Default, Clone, Copy)]
#[must_use]
pub enum ListeningCmd {
    #[default]
    Continue,
    Stop,
}

impl From<()> for ListeningCmd {
    fn from(_: ()) -> Self {
        Default::default()
    }
}

impl BitOr for ListeningCmd {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ListeningCmd::Continue, ListeningCmd::Continue) => ListeningCmd::Continue,
            _ => ListeningCmd::Stop,
        }
    }
}

impl BitOrAssign for ListeningCmd {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

/// Stores the key bindings and actions associated
/// to a specific keyboard.
pub struct KeyboardTriggers {
    keyboard_name: String,
    triggers: Vec<Trigger>,
    current_pressed: Option<usize>,
    take_ownership: bool,
}

impl KeyboardTriggers {
    /// Creates a group of keyboard triggers for the specified
    /// keyboard, disabling the keyboard's original keyboard functionality.
    pub fn new(keyboard_name: impl ToString) -> Self {
        Self {
            keyboard_name: keyboard_name.to_string(),
            triggers: Vec::new(),
            current_pressed: None,
            take_ownership: true,
        }
    }

    /// Creates a group of keyboard triggers for the specified
    /// keyboard that makes the keyboard keep its keyboard functionality.
    pub fn new_keep_keyboard(name: impl ToString) -> Self {
        Self {
            keyboard_name: name.to_string(),
            triggers: Vec::new(),
            current_pressed: None,
            take_ownership: false,
        }
    }

    /// Inserts an action to perform when the provided keys are all pressed.
    pub fn insert<P, C>(&mut self, keys: &[KeyMappingCode], mut on_pressed: P)
    where
        C: Into<ListeningCmd>,
        P: FnMut() -> C + 'static,
    {
        self.insert_boxed(keys, Box::new(move || on_pressed().into()), None)
    }

    /// Inserts an action to perform when the provided keys are all pressed,
    /// and an action to perform when the key combination is released.
    pub fn insert_with_release<P, R, C1, C2>(
        &mut self,
        keys: &[KeyMappingCode],
        mut on_pressed: P,
        mut on_released: R,
    ) where
        C1: Into<ListeningCmd>,
        C2: Into<ListeningCmd>,
        P: FnMut() -> C1 + 'static,
        R: FnMut() -> C2 + 'static,
    {
        self.insert_boxed(
            keys,
            Box::new(move || on_pressed().into()),
            Some(Box::new(move || on_released().into())),
        )
    }

    fn insert_boxed(
        &mut self,
        keys: &[KeyMappingCode],
        on_pressed: Box<dyn FnMut() -> ListeningCmd>,
        on_released: Option<Box<dyn FnMut() -> ListeningCmd>>,
    ) {
        self.triggers.push(Trigger {
            trigger: keys.iter().copied().map(crate::platform::keymap).collect(),
            on_pressed,
            on_released,
        });
    }
}

/// Data structure containing the triggers registered for
/// all the keyboards. Once triggers have been registered,
/// use the [`Triggers::listen`] method to enable the
/// detection.
#[derive(Default)]
pub struct Triggers {
    candidates: HashMap<String, KeyboardTriggers>,
}

impl Triggers {
    /// Registers all the triggers for a specific keyboard,
    /// overriding previous definitions.
    pub fn insert(&mut self, triggers: KeyboardTriggers) {
        self.candidates
            .insert(triggers.keyboard_name.clone(), triggers);
    }

    /// Blocks the process to detect inputs for the configured keyboards
    /// and run the actions associated to key combinations.
    pub fn listen(self) {
        crate::platform::run_input_handler(self);
    }

    pub(crate) fn devices_to_own(&self) -> HashSet<String> {
        self.candidates
            .iter()
            .filter(|(_, v)| v.take_ownership)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Returns whether an action was run, along with a command
    /// tasking the executor to stop or continue.
    pub(crate) fn try_run(&mut self, keyboard: &str, keys: &SetU32) -> (bool, ListeningCmd) {
        let mut triggered = false;
        let mut cmd = ListeningCmd::default();
        if let Some(candidates) = self.candidates.get_mut(keyboard) {
            let mut to_release = None;
            for (i, trigger) in candidates.triggers.iter_mut().enumerate() {
                if *keys == trigger.trigger {
                    triggered = true;
                    to_release = candidates.current_pressed;
                    candidates.current_pressed = Some(i);
                    cmd |= (trigger.on_pressed)();
                    break;
                }
            }

            if let Some(release) = to_release {
                if let Some(release) = &mut candidates
                    .triggers
                    .get_mut(release)
                    .expect("to release trigger not found")
                    .on_released
                {
                    cmd |= release();
                }
            }
        }

        (triggered, cmd)
    }

    pub(crate) fn release(&mut self, keyboard: &str) -> ListeningCmd {
        let mut cmd = ListeningCmd::default();
        if let Some(candidates) = self.candidates.get_mut(keyboard) {
            if let Some(release) = candidates.current_pressed {
                if let Some(release) = &mut candidates
                    .triggers
                    .get_mut(release)
                    .expect("to release trigger not found")
                    .on_released
                {
                    cmd |= release();
                }

                candidates.current_pressed = None;
            }
        }

        cmd
    }
}
