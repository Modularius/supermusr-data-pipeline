use super::NexusGroupDef;

pub(crate) struct NexusGroup<D: NexusGroupDef> {
    name: String,
    definition: D,
}

impl<D: NexusGroupDef> NexusGroup<D> {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            definition: D::new(),
        }
    }
}
