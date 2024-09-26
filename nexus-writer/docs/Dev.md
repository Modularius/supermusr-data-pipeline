# Dev README

## Introduction

The Nexus Writer crate is designed to adhere to 3 goals:

1. To decouple the logic of the Nexus file structure from the details of the HDF5 library.
2. To decouple the logic of handling runs and run messages from the Nexus file structure.
3. To ensure errors in implementing the Nexus file structure are captured at compile time, as far as possible.

To this end the source code is split between the folders `nexus`, `elements` and `schematic`.

# Elements

## NexusGroup

The `NexusGroup` object is a generic structure (i.e. a structure which depends on one or more generic types).
In particular `NexusGroup<G>` is the concrete structure where `G` implements the `NexusGroupDef` trait.

```rust
pub(crate) trait NexusGroupDef: Sized {
    const CLASS_NAME: &'static str;
    type Settings;

    fn new(_settings: &Self::Settings) -> Self;
}
```

Structures implementing `NexusGroupDef` define a `Group` in the nexus file.
That is a `HDF5` group along with a string attribute `NXclass` whose value is statically defined by `NexusGroupDef::CLASS_NAME`.

When calling `NexusGroup::<G>::new(name)` with `name : &str`, `NexusGroup` invokes `NexusGroupDef::new`.
The logic setting up the group should be implemented in `G::new`. For example:

```rust
struct MyGroup {
    //...
}

impl NexusGroupDef for MyGroup {
    const CLASS_NAME: &'static str = "NXMyClass";

    type Settings = ();

    fn new(_settings: &Self::Settings) -> Self {
        Self {
            // Initialise the contents of MyGroup
        }
    }
}
```

So if `my_group : NexusGroup<MyGroup>`,
then calling `my_group.new("MyGroup")` initialises `my_group` to create a `HDF5` group whose structures are defined by the fields of `MyGroup`, and recursively initialised.

Note that the actual HDF5 objects are not created by `new` but created lazily during calls to `NexusGroup::<G>::push_message`.

## NexusDataset

The `NexusDataset` object is a generic structure depending on a type implementing the `NexusDatasetDef` trait. This is analagous to `NexusGroup` and types implementing `NexusGroupDef`. In addition however, `NexusDataset` depends on a type implementing the trait `NexusClassDataHolder`.
This specifies what kind of dataset to create, (e.g. scalar mutable, fixed value, appendable array, etc ).

```rust
pub(crate) trait NexusDatasetDef: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new() -> Self;
}
```

Structures implementing `NexusDatasetDef` define a `HDF5` dataset in the nexus file. There is an optional constant `UNITS` which, if present results in an attribute named `units` specifying the units of the value in the dataset. Many Nexus datasets have this attribute, and if used it should be defined here rather than as a separate attribute.

The purpose of implementing `NexusGroupDef` is to define any attributes the dataset. For instance


```rust
struct MyDataset {
    //...
}

impl NexusDatasetDef for MyGroup {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Seconds);

    fn new() -> Self {
        Self {
            // Initialise any other attributes of MyDataset
        }
    }
}
```

As many Nexus datasets have no attributes, or just a `units` attribute, the unit type `()` also implements `NexusDatasetDef` with `UNITS = None`, and `new()` doing nothing.
So a dataset with no units and other attributes can be specified by `MyDataset : NexusDataset<(),C>` where `C : NexusClassDataHolder`.

### NexusClassDataHolder

The signature of `NexusDatset` is `NexusDatset<D,C>` where `D : NexusDatasetDef` and `C : NexusClassDataHolder`.
The choice of `D` defines the structure, whilst `C` defines the behavior. The choices of `C` are:

- `NexusClassMutableDataHolder<T>`: use this for a statically typed scalar value that can be set and changed at runtime (note: an initial value must be hard-coded).
- `NexusClassFixedDataHolder<T>`: use this for a statically typed scalar value whose value is hard-coded and immutable (e.g. strings with file specifications)
- `NexusClassAppendableDataHolder<T>`: use this for statically typed arrays for which values can be added at runtime.
- `NexusClassNumericAppendableDataHolder`: similar to `NexusClassAppendableDataHolder` except the type is set at runtime and can be any numeric type.

In the above `T : H5Type + Clone + Default` defines the static type where appropriate.

The choice of `C : NexusClassDataHolder` in the signature of `NexusDatset` ensures certain mistakes in behavior are flagged as compiler errors
(such as trying to write to a fixed dataset, or treating an array as a scalar).
The limitation to four types also assists analysis by aiding predictability.

When initialising `NexusDatset` the function to initialise depends on the choice of `C`, as diffent classes expect different initialisation arguments.

- `NexusClassMutableDataHolder`, use either:
  - `NexusDatset::new_with_initial(name: &str, initial : T)`
  - `NexusDatset::new_with_default(name: &str)`
- `NexusClassFixedDataHolder`, use
  - `NexusDatset::new_with_fixed(name: &str, fixed_value : T)`
- `NexusClassAppendableDataHolder`, use either:
  - `NexusDatset::new_with_initial_size(name: &str, default_value: Self::DataType, default_size: usize, chunk_size: usize)`
  - `NexusDatset::new_appendable_with_default(name: &str, chunk_size: usize)`
- `NexusClassNumericAppendableDataHolder`, use
  - `NexusDatset::new(name: &str, chunk_size: usize)`

## NexusAttribute

This is similar to `NexusDataset` except attributes have no internal structure, so there is **no** `NexusAttributeDef` trait.
The signature of `NexusAttribute` is `NexusAttribute<C>` where `C : NexusClassDataHolder`.
Unlike `NexusDataset` however, only `NexusClassMutableDataHolder` and `NexusClassFixedDataHolder` can be chosen.

# Schematic

## Example