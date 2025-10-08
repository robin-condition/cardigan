//pub mod ;

use std::sync::mpsc::Receiver;

pub use cardigan_macros::*;

#[derive(PartialEq, Eq, Clone, Default, Copy)]
pub struct Version {
    id: usize,
}

impl Version {
    const MAX_ID: usize = 1 << 60;

    pub fn first() -> Self {
        Version { id: 0 }
    }
    pub fn next(self) -> Self {
        let last_id = self.id;
        let mut next_id = last_id + 1;
        if next_id >= Self::MAX_ID {
            next_id -= Self::MAX_ID;
        }

        Self { id: next_id }
    }
}

pub struct Versioned<T> {
    value: Option<T>,
    id: Version,
}

impl<T> Default for Versioned<T> {
    fn default() -> Self {
        Self { value: Default::default(), id: Default::default() }
    }
}

impl<T> Versioned<T> {
    pub fn get_value(&self) -> &Option<T> {
        &self.value
    }

    pub fn version(&self) -> &Version {
        &self.id
    }

    pub fn next(self, value: Option<T>) -> Versioned<T> {
        return Versioned {
            value,
            id: self.id.next(),
        };
    }

    pub fn set_to_next(&mut self, value: Option<T>) {
        self.value = value;
        self.id = self.id.next();
    }

    pub fn map<K>(&self, f: impl FnOnce(&Option<T>) -> Option<K>) -> Versioned<K> {
        Versioned { value: f(&self.value), id: self.id }
    }

    pub fn mapmap<K>(&self, f: impl FnOnce(&T) -> K) -> Versioned<K> {
        Versioned { value: match &self.value {
            Some(r) => Some(f(r)),
            None => None,
        }, id: self.id }
    }

    pub fn my_as_ref(&self) -> Versioned<&T> {
        Versioned { value: self.value.as_ref(), id: self.id }
    }
}

impl<T: PartialEq> Versioned<T> {
    pub fn set_to_next_if_unequal(&mut self, val: Option<T>) {
        if val != *self.get_value() {
            self.set_to_next(val);
        }
    }
}

pub struct VersionedInputs<const ARG_COUNT: usize> {
    input_versions: [Version; ARG_COUNT],
}

impl<const ARG_COUNT: usize> Default for VersionedInputs<ARG_COUNT> {
    fn default() -> Self {
        Self { input_versions: [Default::default(); ARG_COUNT] }
    }
}

impl<const ARG_COUNT: usize> VersionedInputs<ARG_COUNT> {
    pub fn must_recompute(&self, inputs: &[Version; ARG_COUNT]) -> bool {
        self.input_versions
            .iter()
            .enumerate()
            .any(|(i, f)| *f != inputs[i])
    }

    pub fn update(&mut self, inputs: &[Version; ARG_COUNT]) {
        self.input_versions = inputs.clone()
    }

    pub fn check_and_update(&mut self, inputs: &[Version; ARG_COUNT]) -> bool {
        let res = self.must_recompute(inputs);
        if res {
            self.update(inputs);
        }
        res
    }
}

#[derive(Default)]
pub struct GeneralVersionedComp<const ARG_COUNT: usize> {
    input_versions: VersionedInputs<ARG_COUNT>,
    output_version: Version
}

impl<const ARG_COUNT: usize> GeneralVersionedComp<ARG_COUNT> {
    pub fn check_and_update(&mut self, inputs: &[Version; ARG_COUNT]) -> bool {
        if self.input_versions.check_and_update(inputs) {
            self.output_version = self.output_version.next();
            return true;
        }
        return false;
    }

    pub fn get_version(&self) -> Version {
        self.output_version.clone()
    }
}

pub struct ReceivedVersioned<T> {
    recvr: Receiver<T>,
    versioned: Versioned<T>
}

impl<T> ReceivedVersioned<T> {
    pub fn new_with_none(recvr: Receiver<T>) -> Self {
        Self {
            recvr,
            versioned: Default::default(),
        }
    }

    pub fn get_value(&mut self) -> &Versioned<T> {
        if let Ok(mut val) = self.recvr.try_recv() {
            while let Ok(new_val) = self.recvr.try_recv() {val = new_val;}

            self.versioned.set_to_next(Some(val));
        }
        &self.versioned
    }
}