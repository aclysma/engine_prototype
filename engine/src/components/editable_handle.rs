use atelier_assets::loader::handle::Handle;
use std::ops::{Deref, DerefMut};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_diff::{ApplyContext, DiffContext, SerdeDiff};
use imgui_inspect::InspectArgsDefault;
use imgui::Ui;


#[derive(Eq)]
pub struct EditableHandle<T: ?Sized> {
    pub handle: Handle<T>
}

impl<T: ?Sized> From<Handle<T>> for EditableHandle<T> {
    fn from(handle: Handle<T>) -> Self {
        EditableHandle {
            handle
        }
    }
}

// impl<T> EditableHandle<T: ?Sized> {
//
// }

impl<T: ?Sized> PartialEq for EditableHandle<T> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.handle == other.handle
    }
}

impl<T: ?Sized> Clone for EditableHandle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<T> std::fmt::Debug for EditableHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditableHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T> Deref for EditableHandle<T> {
    type Target = Handle<T>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T> DerefMut for EditableHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl<T> Serialize for EditableHandle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        self.handle.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for EditableHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let handle = <Handle<T> as Deserialize>::deserialize(deserializer)?;
        Ok(EditableHandle {
            handle
        })
    }
}

impl<T> SerdeDiff for EditableHandle<T> {
    fn diff<'a, S: serde::ser::SerializeSeq>(&self, ctx: &mut DiffContext<'a, S>, other: &Self) -> Result<bool, <S as serde::ser::SerializeSeq>::Error> {
        if self.handle != other.handle {
            ctx.save_value(other)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn apply<'de, A>(&mut self, seq: &mut A, ctx: &mut ApplyContext) -> Result<bool, <A as serde::de::SeqAccess<'de>>::Error> where
        A: serde::de::SeqAccess<'de> {
        ctx.read_value(seq, self)
    }
}

impl<T> imgui_inspect::InspectRenderDefault<EditableHandle<T>> for EditableHandle<T> {
    fn render(data: &[&EditableHandle<T>], _label: &'static str, ui: &Ui, _args: &InspectArgsDefault) {
        ui.text(imgui::im_str!("handle test output {:?}", data[0].handle));
    }

    fn render_mut(data: &mut [&mut EditableHandle<T>], _label: &'static str, ui: &Ui, _args: &InspectArgsDefault) -> bool {
        ui.text(imgui::im_str!("handle test output {:?}", data[0].handle));
        false
    }
}