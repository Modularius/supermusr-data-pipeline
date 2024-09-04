use std::{rc::Rc, sync::Mutex};

use hdf5::Group;

pub(crate) mod attribute;
pub(crate) mod dataset;
pub(crate) mod group;
pub(crate) mod traits;

pub(crate) type SmartPointer<T> = Rc<Mutex<T>>;

pub(crate) trait NxLivesInGroup {
    fn create(&mut self, parent: &Group) -> anyhow::Result<()>;
    fn open(&mut self, parent: &Group) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;
}

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::sync::Mutex;

    use crate::schematic::*;

    use super::attribute::*;
    use super::dataset::*;
    use super::group::*;
    use super::traits::*;

    struct CateDataset {
        a1: NexusAttribute<f64>,
        a2: NexusAttributeFixed<f64>,
    }

    impl NxDataset for CateDataset {
        fn new(attribute_register: AttributeRegister) -> Self {
            Self {
                a1: NexusAttribute::begin("URL").finish(&attribute_register),
                a2: NexusAttribute::begin("Height")
                    .fixed_value(64.0)
                    .finish(&attribute_register),
            }
        }
    }

    struct TopGroup {
        g1: NexusGroup<MiddleGroup>,
        g2: NexusGroup<LowerGroup>,
        d1: NexusDataset<H5String>,
        d2: NexusDatasetResize<bool, CateDataset>,
    }

    impl NxGroup for TopGroup {
        const CLASS_NAME: &'static str = "HiClass";

        fn new(content_register: GroupContentRegister) -> Self {
            Self {
                g1: NexusGroup::new_subgroup("Alfred", &content_register),
                g2: NexusGroup::new_subgroup("Alfred", &content_register),
                d1: NexusDataset::begin("Beth").finish(&content_register),
                d2: NexusDataset::begin("Cate")
                    .resizable(80, 800)
                    .finish(&content_register),
            }
        }
    }

    struct BethDataset {
        a1: NexusAttribute<H5String>,
    }
    impl NxDataset for BethDataset {
        const UNITS: Option<NexusUnits> = Some(NexusUnits::MegaElectronVolts);

        fn new(attribute_register: AttributeRegister) -> Self {
            Self {
                a1: NexusAttribute::begin("NHS Number").finish(&attribute_register),
            }
        }
    }

    struct MiddleGroup {
        g1: NexusGroup<LowerGroup>,
        d1: NexusDatasetFixed<H5String, BethDataset>,
    }

    impl NxGroup for MiddleGroup {
        const CLASS_NAME: &'static str = "MiddleClass";

        fn new(content_register: GroupContentRegister) -> Self {
            Self {
                g1: NexusGroup::new_subgroup("Alfred", &content_register),
                d1: NexusDataset::begin("Beth")
                    .fixed_value("Beth's Name".parse().unwrap())
                    .finish(&content_register),
            }
        }
    }

    struct LowerGroup {
        d1: NexusDataset<H5String>,
        d2: NexusDatasetFixed<bool>,
        d3: NexusDatasetResize<i32>,
    }

    impl NxGroup for LowerGroup {
        const CLASS_NAME: &'static str = "LowClass";

        fn new(content_register: GroupContentRegister) -> Self {
            Self {
                d1: NexusDataset::begin("Alfred").finish(&content_register),
                d2: NexusDataset::begin("Beth")
                    .fixed_value(true)
                    .finish(&content_register),
                d3: NexusDataset::begin("Callum")
                    .resizable(84, 455)
                    .finish(&content_register),
            }
        }
    }

    #[test]
    fn create() {
        let root = NexusGroup::<TopGroup>::new_toplevel("root");
        assert!(root.is_name("root"));

        assert_eq!(root.examine_children(|c| c.len()), 4);
        assert_eq!(root.examine(|x| x.g1.examine_children(|c| c.len())), 2);
        assert_eq!(root.examine(|x| x.g2.examine_children(|c| c.len())), 3);

        assert_eq!(
            root.examine(|x| x.g1.examine(|y| y.g1.examine_children(|c| c.len()))),
            3
        );
    }
}
