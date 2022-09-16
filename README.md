# SpeedTypeTP

![rust workflow](https://github.com/codecontrollers/speedtype-text-processor/actions/workflows/ci.yml/badge.svg)

SpeedTypeTP is a high-speed text processor for the SpeedType project written in Rust. 

It extracts words and their frequencies from large quantities of text files while discarding "implausible" words based on a set of rules.
Parallel loops are used to speed things up. Processing all of English Wikipedia (~14GB) takes less than 3 minutes on a Ryzen 5900x.

> **Note** This software was designed for English words ONLY. Also, the extracted words aren't "complete". 
The rules used to clean up the raw date include the removal of Roman numerals among other things, which causes "false positives" to be removed as well.
