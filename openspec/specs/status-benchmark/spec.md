## Purpose
Define the benchmark fixture and measurement behavior for full `git-vmr status` command performance.

## Requirements

### Requirement: Benchmark measures full status command wall time
The benchmark SHALL measure the wall-clock duration of invoking the compiled `git-vmr` binary with `-C <fixture-root> status`.

#### Scenario: Command invocation is measured
- **WHEN** the status benchmark runs measured iterations
- **THEN** each measured iteration SHALL execute `git-vmr -C <fixture-root> status` as a subprocess

#### Scenario: Setup time is excluded
- **WHEN** the benchmark creates or validates the fixture before measurement
- **THEN** clone, checkout, and validation time SHALL NOT be included in the measured status command duration

### Requirement: Benchmark uses a retained 50-repository real-world fixture
The benchmark SHALL use a VMR fixture containing 50 known real-world git repositories as immediate child directories of the fixture root.

#### Scenario: Fixture is missing
- **WHEN** the benchmark fixture does not exist or one of the configured repositories is missing
- **THEN** the benchmark SHALL clone the missing repositories and prepare the VMR fixture before measurement

#### Scenario: Fixture already exists
- **WHEN** all configured repositories already exist in the benchmark fixture
- **THEN** the benchmark SHALL reuse the existing repositories without cloning them again

#### Scenario: Fixture is retained outside source control
- **WHEN** the benchmark prepares cloned repositories
- **THEN** the cloned repositories SHALL be stored in an ignored generated-artifact location by default

### Requirement: Benchmark uses stable repository inputs
The benchmark SHALL define the 50 benchmark repositories in a stable manifest with repository names, clone URLs, and pinned revisions.

#### Scenario: Repository is cloned for the first time
- **WHEN** the benchmark clones a configured repository
- **THEN** it SHALL check out the repository at its configured pinned revision before measurement

#### Scenario: Existing repository is reused
- **WHEN** the benchmark finds an existing configured repository
- **THEN** it SHALL NOT pull or otherwise update that repository from the network as part of normal benchmark execution

### Requirement: Benchmark measures warm-cache performance
The benchmark SHALL run one or more unmeasured warmup invocations before collecting measured samples.

#### Scenario: Warmup runs before sampling
- **WHEN** the fixture is ready and before measured iterations begin
- **THEN** the benchmark SHALL execute the status command at least once without recording that duration as a measured sample

#### Scenario: Benchmark output is discarded during timing
- **WHEN** the benchmark executes warmup or measured status commands
- **THEN** stdout and stderr from the status command SHALL be discarded or captured so terminal rendering does not dominate the benchmark result

### Requirement: Benchmark setup failures are clear
The benchmark SHALL fail with an actionable error when it cannot prepare or validate the 50-repository fixture.

#### Scenario: Repository clone fails
- **WHEN** a configured repository cannot be cloned during fixture setup
- **THEN** the benchmark SHALL fail and identify the repository that could not be prepared

#### Scenario: Fixture validation fails
- **WHEN** the fixture does not contain a `.gitvmr/` marker and 50 configured immediate child git repositories after setup
- **THEN** the benchmark SHALL fail before measured iterations begin
