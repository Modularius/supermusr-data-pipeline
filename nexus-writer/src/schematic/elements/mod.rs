use dataset::NxContainerAttributes;
use hdf5::{Group, Location};

pub(crate) mod attribute;
pub(crate) mod dataset;
pub(crate) mod group;

pub(crate) trait NxLivesInGroup {
    fn create(&mut self, parent: &Group);
    fn open(&mut self, parent: &Group);
    fn close(&mut self);
}

// Nexus Value Options
pub(crate) trait FixedValueOption: Clone {}

#[derive(Clone)]
pub(crate) struct MustEnterFixedValue {}
impl FixedValueOption for MustEnterFixedValue {}

#[derive(Clone)]
pub(crate) struct NoFixedValueNeeded {}
impl FixedValueOption for NoFixedValueNeeded {}

/*
pub(crate) trait AttributesOption: Clone {
    type Attributes: NxContainerAttributes;
}

#[derive(Clone)]
pub(crate) struct MustEnterAttributes<A : NxContainerAttributes> {}
impl<A : NxContainerAttributes> AttributesOption for MustEnterAttributes<A> {
    type Attributes = A;
}

#[derive(Clone)]
pub(crate) struct NoAttributesNeeded {}
impl AttributesOption for NoAttributesNeeded {
    type Attributes = ();
}
 */
