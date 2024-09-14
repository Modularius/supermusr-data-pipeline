# Maintaining Nexus-Writer

## Introduction

## HDF5

## NeXus

### File Structure

The file structure of a Nexus file is defined by the rust source files in the `src/groups` folder.
The directory structure mostly matches the hdf5 group structure of the file format.
For instance the top level hdf5 group is named after the file name and is defined in `src/groups/mod.rs`.
It contains a single group named `raw_data_1`, defined in `src/groups/raw_data/mod.rs`.

### Groups

A Nexus group is a hdf5 group with an attribute `NXclass`.
Nexus groups are defined by structs containing fields of type `NexusDataset` and `NexusGroup`, and which implement the `NexusGroupDef` trait.

A `NexusGroup` struct instantiates a Nexus group in the following way:

```rust
struct MyGroup {
    an_int_field : NexusDataset<i32>
}

impl NexusGroupDef for MyGroup {
    type Settings = ();

    fn new(_settings : &()) -> Self {
        MyGroup {
            an_int_field : NexusDataset::begin("Some Old Integer").finish_with_auto_default(),
        }
    }
}

let my_group = NexusGroup<MyGroup>::new(&());
```

### Datasets

A Nexus dataset is a hdf5 dataset.
