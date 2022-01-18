use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize,
};

use super::{Changeset, Instruction, Op};

/// Serializes instructions from a [super::Changeset] in a way that is easier to parse on the JS
/// side than what would be optained using the serde derive macros.
pub(crate) struct InstructionSerializer<'a>(pub(crate) Changeset<'a>);

impl<'a> Serialize for InstructionSerializer<'a> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = s.serialize_seq(Some(self.0.ops.len()))?;
        for op in self.0.ops.iter() {
            seq.serialize_element(&OpSerializer(op))?;
        }
        seq.end()
    }
}

struct OpSerializer<'a, 'b>(&'b Op<'a>);

impl<'a, 'b> Serialize for OpSerializer<'a, 'b> {
    // todo: use acronyms for types of operation to make size smaller
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let blank_string = &"".to_string();
        let mut struct_serializer = serializer.serialize_struct("Instruction", 3)?;
        struct_serializer.serialize_field("el", &self.0.id)?;
        match &self.0.instruction {
            Instruction::InsertChild { new_child_id } => {
                struct_serializer.serialize_field("ty", "insertChild")?;
                struct_serializer.serialize_field("payload", &new_child_id)?;
            }
            Instruction::InsertAfter { after_id } => {
                struct_serializer.serialize_field("ty", "insertAfter")?;
                struct_serializer.serialize_field("payload", &after_id)?;
            }
            Instruction::SetAttribute { key, value } => {
                struct_serializer.serialize_field("ty", "setAttr")?;
                struct_serializer.serialize_field(
                    "payload",
                    &format!("{key}+{value}", key = key, value = value),
                )?;
            }
            Instruction::SetId { value } => {
                struct_serializer.serialize_field("ty", "setId")?;
                struct_serializer.serialize_field("payload", &value)?;
            }
            Instruction::SetText { value } => {
                struct_serializer.serialize_field("ty", "setText")?;
                struct_serializer.serialize_field("payload", &value)?;
            }
            Instruction::SetTagName { name } => {
                struct_serializer.serialize_field("ty", "setTagName")?;
                struct_serializer.serialize_field("payload", &name)?;
            }
            Instruction::CreateTag { name, parent_id } => {
                struct_serializer.serialize_field("ty", "createTag")?;
                struct_serializer.serialize_field(
                    "payload",
                    &format!(
                        "{}+{}",
                        name,
                        if let Some(p) = parent_id {
                            p
                        } else {
                            blank_string
                        }
                    ),
                )?;
            }
            Instruction::RemoveText => {
                struct_serializer.serialize_field("ty", "removeText")?;
                struct_serializer.skip_field("payload")?;
            }
            Instruction::RemoveListeners => {
                struct_serializer.serialize_field("ty", "removeListeners")?;
                struct_serializer.skip_field("payload")?;
            }
            Instruction::AttachListener { name, on } => {
                struct_serializer.serialize_field("ty", "attachListener")?;
                struct_serializer.serialize_field("payload", &format!("{}+{}", name, on))?;
            }
            Instruction::SetInnerHtml { element, html } => {
                struct_serializer.serialize_field("ty", "setInnerHtml")?;
                struct_serializer.serialize_field("payload", &format!("{}+{}", element, html))?;
            }
            Instruction::InsertBefore { before_id } => {
                struct_serializer.serialize_field("ty", "insertBefore")?;
                struct_serializer.serialize_field("payload", &before_id)?;
            }
            Instruction::RemoveAttribute { key } => {
                struct_serializer.serialize_field("ty", "removeAttr")?;
                struct_serializer.serialize_field("payload", key)?;
            }
        }
        struct_serializer.end()
    }
}
