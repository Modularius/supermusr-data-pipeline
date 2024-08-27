use std::{rc::Rc, sync::Mutex};

use hdf5::{
    types::{TypeDescriptor, VarLenAscii},
    Group,
};

use super::{dataset::NexusDataset, NxLivesInGroup};

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
    
    pub(crate) fn get_group(&self) -> Option<&Group> {
        self.group.as_ref()
    }

    pub(crate) fn validate_self(&self) -> bool {
        if let Some(group) = &self.group {
            let class_name: VarLenAscii = group
                .attr("NXclass")
                .expect("Class Exists")
                .read_scalar()
                .expect("Read Okay");
            group.name().cmp(&self.name).is_eq() && class_name.as_str().cmp(G::CLASS_NAME).is_eq()
        } else {
            true
        }
    }
}

impl<G: NxGroup> NxLivesInGroup for NexusGroup<G> {
    fn create(&mut self, parent: &Group) {
        match parent.create_group(&self.name) {
            Ok(group) => {
                group
                    .new_attr_builder()
                    .with_data_as(G::CLASS_NAME, &TypeDescriptor::VarLenAscii)
                    .create("NXclass")
                    .expect("Can write");
                for content in self.content_register.lock().expect("Lock Exists").iter_mut() {
                    content.lock().expect("Lock Exists").create(&group);
                }
                self.group = Some(group)
            }
            Err(e) => panic!("{e}"),
        }
    }

    fn open(&mut self, parent: &Group) {
        if self.group.is_some() {
            panic!("{} group already open", self.name)
        } else {
            match parent.group(&self.name) {
                Ok(group) => {
                    for content in self.content_register.lock().expect("Lock Exists").iter_mut() {
                        content.lock().expect("Lock Exists").open(&group);
                    }
                    self.group = Some(group);
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    fn close(&mut self) {
        if self.group.is_none() {
            panic!("{} group already closed", self.name)
        } else {
            for content in self.content_register.lock().expect("Lock Exists").iter_mut() {
                content.lock().expect("Lock Exists").close();
            }
            self.group = None
        }
    }
}
