use std::{ops::Deref, rc::Rc, sync::{Mutex, MutexGuard}};

use hdf5::{types::VarLenAscii, Group};
use tracing::instrument;

use super::{traits::GroupBuildable, NxLivesInGroup, SmartPointer};
#[cfg(test)]
use super::traits::Examine;

type GroupContentRegisterContentType = SmartPointer<dyn NxLivesInGroup>;

#[derive(Clone)]
pub(crate) struct GroupContentRegister(SmartPointer<Vec<GroupContentRegisterContentType>>);

impl GroupContentRegister {
    pub(crate) fn new(vec: Vec<GroupContentRegisterContentType>) -> Self {
        GroupContentRegister(Rc::new(Mutex::new(vec)))
    }

    pub(crate) fn apply_lock(&self) -> MutexGuard<'_,Vec<GroupContentRegisterContentType>> {
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

pub(crate) type NexusGroup<G> = SmartPointer<UnderlyingNexusGroup<G>>;

pub(crate) struct UnderlyingNexusGroup<G: NxGroup> {
    name: String,
    class: G,
    group: Option<Group>,
    content_register: GroupContentRegister,
}

impl<G: NxGroup + 'static> GroupBuildable for NexusGroup<G> {
    #[instrument(skip_all, level = "debug", fields(name = name, class = G::CLASS_NAME))]
    fn new_toplevel(name: &str) -> Self {
        let content_register = GroupContentRegister::new(Vec::new().into());
        Rc::new(Mutex::new(UnderlyingNexusGroup::<G> {
            name: name.to_owned(),
            class: G::new(content_register.clone()),
            group: None,
            content_register,
        }))
    }

    #[instrument(skip_all, level = "debug", fields(name = name, class = G::CLASS_NAME))]
    fn new_subgroup(name: &str, parent_content_register: &GroupContentRegister) -> Self {
        let content_register = GroupContentRegister::new(Vec::new().into());
        let rc = Rc::new(Mutex::new(UnderlyingNexusGroup {
            name: name.to_owned(),
            class: G::new(content_register.clone()),
            group: None,
            content_register,
        }));
        parent_content_register
            .lock()
            .push(rc.clone());
        rc
    }

    fn is_name(&self, name: &str) -> bool {
        self.lock().expect("").name == name
    }
}

#[cfg(test)]
impl<G: NxGroup> Examine<Rc<Mutex<dyn NxLivesInGroup>>, G> for NexusGroup<G> {
    fn examine<F, T>(&self, f: F) -> T
    where
        F: Fn(&G) -> T {
            f(&self.lock().unwrap().class)
        }

    fn examine_children<F, T>(&self, f: F) -> T
    where
        F: Fn(&[Rc<Mutex<dyn NxLivesInGroup>>]) -> T {
            f(&self.lock().unwrap().content_register.lock().unwrap())
        }
}

impl<G: NxGroup> NxLivesInGroup for UnderlyingNexusGroup<G> {
    #[instrument(skip_all, level = "debug", fields(name = self.name, class = G::CLASS_NAME), err(level = "error"))]
    fn create(&mut self, parent: &Group) -> anyhow::Result<()> {
        if self.group.is_some() {
            Err(anyhow::anyhow!("{} group already open", self.name))
        } else {
            match parent.create_group(&self.name) {
                Ok(group) => {
                    group
                        .new_attr_builder()
                        .with_data(&[VarLenAscii::from_ascii(G::CLASS_NAME).expect("")])
                        .create("NXclass")
                        .expect("Can write");

                    for content in self
                        .content_register
                        .lock()
                        .iter_mut()
                    {
                        content.lock().expect("Lock Exists").create(&group)?;
                    }
                    self.group = Some(group);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name, class = G::CLASS_NAME), err(level = "error"))]
    fn open(&mut self, parent: &Group) -> anyhow::Result<()> {
        if self.group.is_some() {
            Err(anyhow::anyhow!("{} group already open", self.name))
        } else {
            let group = parent.group(&self.name)?;
            for content in self
                .content_register
                .lock()
                .iter_mut()
            {
                content.lock().expect("Lock Exists").open(&group)?;
            }
            self.group = Some(group);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name, class = G::CLASS_NAME), err(level = "error"))]
    fn close(&mut self) -> anyhow::Result<()> {
        if self.group.is_none() {
            Err(anyhow::anyhow!("{} group already closed", self.name))
        } else {
            for content in self
                .content_register
                .lock()
                .iter_mut()
            {
                content.lock().expect("Lock Exists").close()?;
            }
            self.group = None;
            Ok(())
        }
    }
}

impl<G: NxGroup + NxPushMessage<T, MessageType = T>, T> NxPushMessage<T> for NexusGroup<G> {
    type MessageType = T;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.lock().expect("").class.push_message(message)
    }
}

impl<G: NxGroup + NxPushMessageMut<T, MessageType = T>, T> NxPushMessageMut<T> for NexusGroup<G> {
    type MessageType = T;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.lock().expect("").class.push_message_mut(message)
    }
}
