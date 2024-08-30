use std::{rc::Rc, sync::Mutex};

use hdf5::{types::VarLenAscii, Group};
use tracing::instrument;

use super::NxLivesInGroup;

pub(crate) type GroupContentRegister = Rc<Mutex<Vec<Rc<Mutex<dyn NxLivesInGroup>>>>>;

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

pub(crate) type NexusGroup<G> = Rc<Mutex<UnderlyingNexusGroup<G>>>;

pub(crate) struct UnderlyingNexusGroup<G: NxGroup> {
    name: String,
    class: G,
    group: Option<Group>,
    content_register: GroupContentRegister,
}

pub(crate) trait GroupBuildable {
    fn new_toplevel(name: &str) -> Self;
    fn new_subgroup(name: &str, parent_content_register: &GroupContentRegister) -> Self;
    fn is_name(&self, name: &str) -> bool;
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
    fn new_subgroup(
        name: &str,
        parent_content_register: &GroupContentRegister,
    ) -> Self {
        let content_register = GroupContentRegister::new(Vec::new().into());
        let rc = Rc::new(Mutex::new(UnderlyingNexusGroup {
            name: name.to_owned(),
            class: G::new(content_register.clone()),
            group: None,
            content_register,
        }));
        parent_content_register
            .lock()
            .expect("Lock Exists")
            .push(rc.clone());
        rc
    }

    fn is_name(&self, name: &str) -> bool {
        self.lock().expect("").name == name
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
                        .expect("Lock Exists")
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
            match parent.group(&self.name) {
                Ok(group) => {
                    for content in self
                        .content_register
                        .lock()
                        .expect("Lock Exists")
                        .iter_mut()
                    {
                        content.lock().expect("Lock Exists").open(&group)?;
                    }
                    self.group = Some(group);
                    Ok(())
                }
                Err(e) => panic!("{e}"),
            }
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
                .expect("Lock Exists")
                .iter_mut()
            {
                content.lock().expect("Lock Exists").close()?;
            }
            self.group = None;
            Ok(())
        }
    }
}

impl<G: NxGroup + NxPushMessage<T, MessageType = T>, T> NxPushMessage<T>
    for NexusGroup<G>
{
    type MessageType = T;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.lock().expect("").class.push_message(message)
    }
}

impl<G: NxGroup + NxPushMessageMut<T, MessageType = T>, T> NxPushMessageMut<T>
    for NexusGroup<G>
{
    type MessageType = T;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.lock().expect("").class.push_message_mut(message)
    }
}
