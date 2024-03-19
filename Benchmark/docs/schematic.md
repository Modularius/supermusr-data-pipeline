The following diagram illustrates how the tests are run.
Event Formation: this is the actual tool being tested. It is almost exactly how it appears in the data pipeline.
Events Analysis: This tool matches the simulated events list with the corresponding detected events list and compares them.

```mermaid
sequenceDiagram
box Black Trace Simulator
    participant T as Template
    participant R as Trace RNG
end

note over T: Generates N <br>templates, with some <br>parameter depending<br>on j as 1 <= j <= N.

box Black Test i (i = 1,2,3): Each event formation instance is run with distinct settings.<br>The event analysis instancs are run identically however.
    participant TE as Event Formation[i]
    participant EA as Events Analyser[i]
    participant F as File[i]
end
loop j = 1,...,N
    T->>R: Template j
    
    note over R: Generate R instances<br>of trace template j.
    loop R repeatitions
        R->>TE: Trace
        TE->>EA: Formation Time
        TE->>EA: Detected Events List
        R->>EA: Simulated Events List
    end
    EA->>F: Analysis for Template j
end
```

For each events list, the Event Analyser collates the number of events and the estimated muon lifetime from the data. For each repeated instance of template j, these statistics
