pub(crate) mod dataset;
//mod group;
pub(crate) mod attribute;


pub(crate) trait NxGroup {
    const CLASS_NAME : &'static str;

    fn new() -> Self;
}


pub(crate) struct NexusGroup<G : NxGroup> {
    name: String,
    group : G,
}

impl<G : NxGroup> NexusGroup<G> {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            group: G::new()
        }
    }
}
