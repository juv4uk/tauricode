//! M2.1 — generic runtime handle + lifecycle state.
//!
//! Deliberately a separate axis from `types::Status` (identity
//! freshness/liveness). Conflating "is this process alive" with "is the
//! self-reported identity fresh" was already flagged as a mistake in
//! `ecosystem-observer`'s own docs (`IdentityStatus` is a
//! correlation/liveness status, not a trust/authenticity one) — this
//! module keeps process lifecycle as its own thing rather than repeating
//! that conflation one layer up.

use crate::types::RuntimeHandle;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Launching,
    Running,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    UnknownHandle,
    IllegalTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
}

/// Legal transitions only: `Launching -> Running`, `Running -> Stopped`.
/// Anything else (including re-registering an already-known handle, or
/// transitioning a `Stopped` handle anywhere) is rejected with a typed
/// error rather than silently allowed — matches root policy's "never
/// hide unknown/failure" and this crate's own `AdapterError` convention
/// (M2.3).
#[derive(Debug, Default)]
pub struct HandleRegistry {
    states: HashMap<RuntimeHandle, LifecycleState>,
}

impl HandleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handle: RuntimeHandle) -> Result<(), TransitionError> {
        if self.states.contains_key(&handle) {
            return Err(TransitionError::IllegalTransition {
                from: *self.states.get(&handle).unwrap(),
                to: LifecycleState::Launching,
            });
        }
        self.states.insert(handle, LifecycleState::Launching);
        Ok(())
    }

    pub fn transition(
        &mut self,
        handle: &RuntimeHandle,
        to: LifecycleState,
    ) -> Result<(), TransitionError> {
        let current = *self
            .states
            .get(handle)
            .ok_or(TransitionError::UnknownHandle)?;

        let legal = matches!(
            (current, to),
            (LifecycleState::Launching, LifecycleState::Running)
                | (LifecycleState::Running, LifecycleState::Stopped)
        );

        if !legal {
            return Err(TransitionError::IllegalTransition { from: current, to });
        }

        self.states.insert(handle.clone(), to);
        Ok(())
    }

    pub fn get(&self, handle: &RuntimeHandle) -> Option<LifecycleState> {
        self.states.get(handle).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_transition_sequence_succeeds() {
        let mut registry = HandleRegistry::new();
        let handle = RuntimeHandle::new("h1");

        registry.register(handle.clone()).unwrap();
        assert_eq!(registry.get(&handle), Some(LifecycleState::Launching));

        registry
            .transition(&handle, LifecycleState::Running)
            .unwrap();
        assert_eq!(registry.get(&handle), Some(LifecycleState::Running));

        registry
            .transition(&handle, LifecycleState::Stopped)
            .unwrap();
        assert_eq!(registry.get(&handle), Some(LifecycleState::Stopped));
    }

    #[test]
    fn illegal_transition_is_rejected() {
        let mut registry = HandleRegistry::new();
        let handle = RuntimeHandle::new("h2");
        registry.register(handle.clone()).unwrap();

        // Launching -> Stopped directly is not a legal transition.
        let err = registry
            .transition(&handle, LifecycleState::Stopped)
            .unwrap_err();
        assert_eq!(
            err,
            TransitionError::IllegalTransition {
                from: LifecycleState::Launching,
                to: LifecycleState::Stopped,
            }
        );

        // Confirm the illegal attempt did not mutate state.
        assert_eq!(registry.get(&handle), Some(LifecycleState::Launching));
    }

    #[test]
    fn stopped_handle_cannot_transition_further() {
        let mut registry = HandleRegistry::new();
        let handle = RuntimeHandle::new("h3");
        registry.register(handle.clone()).unwrap();
        registry
            .transition(&handle, LifecycleState::Running)
            .unwrap();
        registry
            .transition(&handle, LifecycleState::Stopped)
            .unwrap();

        let err = registry
            .transition(&handle, LifecycleState::Running)
            .unwrap_err();
        assert_eq!(
            err,
            TransitionError::IllegalTransition {
                from: LifecycleState::Stopped,
                to: LifecycleState::Running,
            }
        );
    }

    #[test]
    fn unknown_handle_lookup_returns_none_not_panic() {
        let registry = HandleRegistry::new();
        let handle = RuntimeHandle::new("never-registered");
        assert_eq!(registry.get(&handle), None);
    }

    #[test]
    fn unknown_handle_transition_returns_typed_error_not_panic() {
        let mut registry = HandleRegistry::new();
        let handle = RuntimeHandle::new("never-registered");
        let err = registry
            .transition(&handle, LifecycleState::Running)
            .unwrap_err();
        assert_eq!(err, TransitionError::UnknownHandle);
    }

    #[test]
    fn double_register_is_rejected() {
        let mut registry = HandleRegistry::new();
        let handle = RuntimeHandle::new("h4");
        registry.register(handle.clone()).unwrap();
        let err = registry.register(handle).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }
}
