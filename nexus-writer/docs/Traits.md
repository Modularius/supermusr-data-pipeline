```mermaid
erDiagram


NexusDataHolderClass["(NexusDataHolder)"] {

}

NexusBuilder {
    C NexusDataHolderClass,
    H NexusDataHolder,
}

NexusBuilderBegun["(BuilderBegun)"] {

}

NexusDataHolder["(NexusDataHolder)"] {
    C NexusDataHolderClass
}

NexusTypedDataHolder["(NexusTypedDataHolder)"] {
    
}

NexusDataset {

}

NexusAttribute {

}
```