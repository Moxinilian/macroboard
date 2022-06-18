use std::collections::{HashMap, HashSet};

use keycode::KeyMappingCode;
use tinyset::SetU32;

struct Trigger {
    trigger: SetU32,
    on_pressed: Box<dyn FnMut()>,
    on_released: Option<Box<dyn FnMut()>>,
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
    pub fn insert<P>(&mut self, keys: &[KeyMappingCode], on_pressed: P)
    where
        P: FnMut() + 'static,
    {
        self.insert_boxed(keys, Box::new(on_pressed), None)
    }

    /// Inserts an action to perform when the provided keys are all pressed,
    /// and an action to perform when the key combination is released.
    pub fn insert_with_release<P, R>(&mut self, keys: &[KeyMappingCode], on_pressed: P, on_released: R)
    where
        P: FnMut() + 'static,
        R: FnMut() + 'static,
    {
        self.insert_boxed(keys, Box::new(on_pressed), Some(Box::new(on_released)))
    }

    fn insert_boxed(
        &mut self,
        keys: &[KeyMappingCode],
        on_pressed: Box<dyn FnMut()>,
        on_released: Option<Box<dyn FnMut()>>,
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

    /// Returns whether an action was run.
    pub(crate) fn try_run(&mut self, keyboard: &str, keys: &SetU32) -> bool {
        let mut triggered = false;
        if let Some(candidates) = self.candidates.get_mut(keyboard) {
            let mut to_release = None;
            for (i, trigger) in candidates.triggers.iter_mut().enumerate() {
                if *keys == trigger.trigger {
                    triggered = true;
                    to_release = candidates.current_pressed;
                    candidates.current_pressed = Some(i);
                    (trigger.on_pressed)();
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
                    release();
                }
            }
        }

        triggered
    }

    pub(crate) fn release(&mut self, keyboard: &str) {
        if let Some(candidates) = self.candidates.get_mut(keyboard) {
            if let Some(release) = candidates.current_pressed {
                if let Some(release) = &mut candidates
                    .triggers
                    .get_mut(release)
                    .expect("to release trigger not found")
                    .on_released
                {
                    release();
                }

                candidates.current_pressed = None;
            }
        }
    }
}
