use std::{rc::Rc, sync::Mutex};

use hdf5::{
    types::{TypeDescriptor, VarLenAscii},
    Group,
};
use tracing::{info, instrument};

use super::NxLivesInGroup;

/*
pub(crate) enum ClassName {
    Entry,
    EventData,
    Instrument,
    Detector,
    Source,
    Period,
    Runlog,
    Log,
    Selog,
    Seblock,
}

impl ClassName {
    fn get(&self) -> &'static str {
        match self {
            ClassName::Entry => "NXentry",
            ClassName::EventData => "NXevent_data",
            ClassName::Instrument => "NXinstrument",
            ClassName::Detector => "NXdetector",
            ClassName::Source => "NXsource",
            ClassName::Period => "NXperiod",
            ClassName::Runlog => "NXrunlog",
            ClassName::Log => "NXlog",
            ClassName::Selog => "IXselog",
            ClassName::Seblock => "IXseblock",
        }
    }
} */

pub(crate) type RcGroupContentRegister = Rc<Mutex<Vec<Rc<Mutex<dyn NxLivesInGroup>>>>>;

pub(crate) trait NxGroup: Sized {
    const CLASS_NAME: &'static str;

    fn new(content_register: RcGroupContentRegister) -> Self;
}

pub(crate) trait NxPushMessage<T> {
    type MessageType;

    fn push_message(&mut self, message: &Self::MessageType);
}

impl<G: NxGroup + NxPushMessage<T, MessageType = T>, T> NxPushMessage<T> for Rc<Mutex<NexusGroup<G>>> {
    type MessageType = T;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.lock().expect("").class.push_message(message)
    }
}

pub(crate) type RcNexusGroup<G> = Rc<Mutex<NexusGroup<G>>>;

pub(crate) struct NexusGroup<G: NxGroup> {
    name: String,
    class: G,
    group: Option<Group>,
    content_register: RcGroupContentRegister,
}

impl<G: NxGroup + 'static> NexusGroup<G> {
    #[instrument(skip_all, level = "debug", fields(name = name, class = G::CLASS_NAME))]
    pub(crate) fn new(name: &str, parent_content_register: Option<RcGroupContentRegister>) -> Rc<Mutex<Self>> {
        let content_register = RcGroupContentRegister::new(Vec::new().into());
        let rc = Rc::new(Mutex::new(Self {
            name: name.to_owned(),
            class: G::new(content_register.clone()),
            group: None,
            content_register,
        }));
        if let Some(parent_content_register) = parent_content_register {
            parent_content_register.lock().expect("Lock Exists").push(rc.clone());
        }
        rc
    }
}

impl<G: NxGroup> NxLivesInGroup for NexusGroup<G> {
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
                    
                    for content in self.content_register.lock().expect("Lock Exists").iter_mut() {
                        content.lock().expect("Lock Exists").create(&group);
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
                    for content in self.content_register.lock().expect("Lock Exists").iter_mut() {
                        content.lock().expect("Lock Exists").open(&group);
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
            for content in self.content_register.lock().expect("Lock Exists").iter_mut() {
                content.lock().expect("Lock Exists").close();
            }
            self.group = None;
            Ok(())
        }
    }
}
