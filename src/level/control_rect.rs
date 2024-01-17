use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use uid::Id;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd,Eq)]
pub struct ControlRectId {
    id: Id<PhantomData<ControlRect>>,
}
impl Deref for ControlRectId {
    type Target = Id<PhantomData<ControlRect>>;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}
impl DerefMut for ControlRectId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}
impl ControlRectId {
    pub fn new() -> Self {
        Self { id: Id::new() }
    }
}
impl Serialize for ControlRectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.clone().get().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ControlRectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            id: unsafe { Id::new_unchecked(usize::deserialize(deserializer)?) },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRect {
    pub position: [f32; 3],
    pub rotation: f32,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl ControlRect {
    pub fn new(position: [f32; 3], rotation: f32, width: f32, height: f32) -> Self {
        Self {
            position,
            rotation,
            top: height / 2.,
            bottom: height / 2.,
            left: width / 2.,
            right: width / 2.,
        }
    }
}


