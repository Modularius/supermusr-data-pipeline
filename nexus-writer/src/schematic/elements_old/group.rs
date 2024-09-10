use std::{
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

use hdf5::{types::VarLenAscii, Group};
use tracing::instrument;

#[cfg(test)]
use super::traits::Examine;
use super::{
    error::{ClosingError, CreationError, HDF5Error, OpeningError},
    traits::{SubgroupBuildable, TopGroupBuildable},
    NxLivesInGroup, SmartPointer,
};

type GroupContentRegisterContentType = SmartPointer<dyn NxLivesInGroup>;

#[derive(Clone)]
pub(crate) struct GroupContentRegister(SmartPointer<Vec<GroupContentRegisterContentType>>);

impl GroupContentRegister {
    pub(crate) fn new(vec: Vec<GroupContentRegisterContentType>) -> Self {
        GroupContentRegister(Rc::new(Mutex::new(vec)))
    }

    pub(crate) fn lock_mutex(&self) -> MutexGuard<'_, Vec<GroupContentRegisterContentType>> {
        self.0.lock().expect("Lock exists")
    }
}

pub(crate) trait NxGroup: Sized {
    const CLASS_NAME: &'static str;

    fn new(content_register: GroupContentRegister) -> Self;
}

pub(crate) trait NxPushMessage<T> {
    type MessageType;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()>;
}

pub(crate) trait NxPushMessageMut<T> {
    type MessageType;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()>;
}

pub(crate) type TopLevelNexusGroup<G> = NexusGroup<G, false>;
pub(crate) struct NexusGroup<G: NxGroup, const IS_SUBGROUP: bool = true>(
    SmartPointer<UnderlyingNexusGroup<G>>,
);

impl<G, const IS_SUBGROUP: bool> NexusGroup<G, IS_SUBGROUP>
where
    G: NxGroup,
{
    fn new_internal(group: UnderlyingNexusGroup<G>) -> Self {
        NexusGroup(Rc::new(Mutex::new(group)))
    }
    pub(crate) fn apply_lock(&self) -> MutexGuard<'_, UnderlyingNexusGroup<G>> {
        self.0.lock().expect("Lock exists")
    }
    fn clone_inner(&self) -> SmartPointer<UnderlyingNexusGroup<G>> {
        self.0.clone()
    }
    pub(crate) fn is_name(&self, name: &str) -> bool {
        self.apply_lock().name == name
    }
}

pub(crate) struct UnderlyingNexusGroup<G: NxGroup> {
    name: String,
    class: G,
    group: Option<Group>,
    content_register: GroupContentRegister,
}

impl<G: NxGroup + 'static> SubgroupBuildable for NexusGroup<G, true> {
    #[instrument(skip_all, level = "debug", fields(name = name, class = G::CLASS_NAME))]
    fn new_subgroup(name: &str, parent_content_register: &GroupContentRegister) -> Self {
        let content_register = GroupContentRegister::new(Vec::new());
        let rc = NexusGroup::new_internal(UnderlyingNexusGroup {
            name: name.to_owned(),
            class: G::new(content_register.clone()),
            group: None,
            content_register,
        });
        parent_content_register.lock_mutex().push(rc.clone_inner());
        rc
    }
}

impl<G: NxGroup + 'static> TopGroupBuildable for NexusGroup<G, false> {
    #[instrument(skip_all, level = "debug", fields(name = name, class = G::CLASS_NAME))]
    fn new_toplevel(name: &str) -> Self {
        let content_register = GroupContentRegister::new(Vec::new());
        NexusGroup::new_internal(UnderlyingNexusGroup::<G> {
            name: name.to_owned(),
            class: G::new(content_register.clone()),
            group: None,
            content_register,
        })
    }
}

#[cfg(test)]
impl<G: NxGroup, const IS_SUBGROUP: bool> Examine<Rc<Mutex<dyn NxLivesInGroup>>, G>
    for NexusGroup<G, IS_SUBGROUP>
{
    fn examine<F, T>(&self, f: F) -> T
    where
        F: Fn(&G) -> T,
    {
        f(&self.apply_lock().class)
    }

    fn examine_children<F, T>(&self, f: F) -> T
    where
        F: Fn(&[Rc<Mutex<dyn NxLivesInGroup>>]) -> T,
    {
        f(&self.apply_lock().content_register.lock_mutex())
    }
}

impl<G: NxGroup> NxLivesInGroup for UnderlyingNexusGroup<G> {
    #[instrument(skip_all, level = "debug", fields(name = self.name, class = G::CLASS_NAME), err(level = "error"))]
    fn create(&mut self, parent: &Group) -> Result<(), CreationError> {
        if self.group.is_some() {
            Err(CreationError::AlreadyOpen)
        } else {
            let group = parent
                .create_group(&self.name)
                .map_err(HDF5Error::General)?;

            group
                .new_attr_builder()
                .with_data(&[VarLenAscii::from_ascii(G::CLASS_NAME).map_err(HDF5Error::String)?])
                .create("NXclass")
                .expect("Can write");

            for content in self.content_register.lock_mutex().iter_mut() {
                content.lock().expect("Lock Exists").create(&group)?;
            }
            self.group = Some(group);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name, class = G::CLASS_NAME), err(level = "error"))]
    fn open(&mut self, parent: &Group) -> Result<(), OpeningError> {
        if self.group.is_some() {
            Err(OpeningError::AlreadyOpen)
        } else {
            let group = parent.group(&self.name).map_err(HDF5Error::General)?;
            for content in self.content_register.lock_mutex().iter_mut() {
                content.lock().expect("Lock Exists").open(&group)?;
            }
            self.group = Some(group);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name, class = G::CLASS_NAME), err(level = "error"))]
    fn close(&mut self) -> Result<(), ClosingError> {
        if self.group.is_none() {
            Err(ClosingError::AlreadyClosed)
        } else {
            for content in self.content_register.lock_mutex().iter_mut() {
                content.lock().expect("Lock Exists").close()?;
            }
            self.group = None;
            Ok(())
        }
    }
}

impl<G: NxGroup + NxPushMessage<T, MessageType = T>, T, const IS_SUBGROUP: bool> NxPushMessage<T>
    for NexusGroup<G, IS_SUBGROUP>
{
    type MessageType = T;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.apply_lock().class.push_message(message)
    }
}

impl<G: NxGroup + NxPushMessageMut<T, MessageType = T>, T, const IS_SUBGROUP: bool>
    NxPushMessageMut<T> for NexusGroup<G, IS_SUBGROUP>
{
    type MessageType = T;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.apply_lock().class.push_message_mut(message)
    }
}
