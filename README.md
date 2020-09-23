sensorlog
========

sensorlog is a lightweight data logging service.

[![Build Status](https://travis-ci.org/nyantec/sensorlog.svg?branch=master)](https://travis-ci.org/nyantec/sensorlog)


Data Model
----------

The interface is very simple and consists of only two main operations:

  - `insertMeasurement(sensor_id, time, data)` - Store a measurement
  - `fetchMeasurements(sensor_id, from, until)` - Retrieve all measurements in the specified time range

A measurement is a 3-tuple of `time`, `sensor_id` and `data`. The `time` field
should contain the wall clock time at which the measurement was taken. The `sensor_id`
field is used to identify a logical stream of consecutive measurements. The `data`
field is treated as an opaque blob; the contents are entirely user-defined.


Installation
------------

Add the library to your `Cargo.toml`:

``` toml
[dependencies]
sensorlog = "1.0.0"
```

Getting Started
---------------

First you need to create a logfile config, describing how the data is stored,
especially the storage quotas (see section "Retention & Quotas" for more):

```rust
let mut logfile_config = sensorlog::logfile_config::LogfileConfig::new();
// This sets the default qoate, i.e. how much storage space a sensor can use
// You can either directly initiate the enum
logfile_config.set_default_storage_quota(sensorlog::quota::StorageQuota::Unlimited);
// Or use the parse_string utility function
logfile_config.set_default_storage_quota(sensorlog::quota::StorageQuota::parse_string("unlimited")?);
```

Then you need to create a `Sensorlog` instance:
```rust
let datadir = PathBuf::from("/tmp/sensordata");
let service = sensorlog::Sensorlog::new(&datadir, logfile_config)?;
```

Now you can insert some data. Run the following code to insert the measurement
"3250" for sensor 's1.hydraulic_pressure_psi' (if the time parameter is `None`,
it will be defaulted to the current wall clock time):

```rust
service.store_measurement(None, "s1.hydraulic_pressure_psi", "3250")?;
```

Afterwards, run this code to retrieve the last 10 minutes of measurements from
the 's1.hydraulic_pressure_psi' sensor:

```rust
let now = sensorlog::time::get_unix_microseconds()?;
let ten_minutes_ago = now - 10 * 60 * 1000000;
let measurements = service.fetch_measurements("s1.hydraulic_pressure_psi", None, Some(ten_minutes_ago), None)?;
println!("Fetched measurements: {:?}", measurements);
```

The output should look similar to this:

```
Fetched measurements: [Measurement { time: 1600872248099558, data: [51, 50, 53, 48] }]
```

Retention & Quotas
------------------

The retention and garbage collection system of sensorlog is based around a simple
storage quota that is assigned to each `sensor_id`. Once the quota for a given
sensor is used up, old measurements are dropped whenever new measurements are
added for that sensor. Quotas are expressed in bytes and rotation of measurements
is always first-in, first-out.

You can set a default quota value that applies to all sensors as well as a quota
override for each individual sensor id.

For example, to use sensorlog in a "allowlist" configuration, set the default
quota to zero and then explicitly allocate storage for each sensor.

```rust
let mut logfile_config = sensorlog::logfile_config::LogfileConfig::new();
logfile_config.set_default_storage_quota(sensorlog::quota::StorageQuota::Zero);

let datadir = PathBuf::from("/tmp/sensordata");
let mut service = sensorlog::Sensorlog::new(&datadir, logfile_config)?;

service.set_storage_quota_for("my.first.key", sensorlog::quota::StorageQuota::parse_string("1MB")?);
service.set_storage_quota_for("some/other/ley", sensorlog::quota::StorageQuota::parse_string("4MB")?);
```

In the above configuration, the total disk space used by sensorlog will be bounded,
but you can not insert data from sensors that are not pre-configured. The exact
opposite configuration would be setting the default quota to infinite. This
configuration allows you to store data from not previously known sensors, but
may use an unbounded amount of disk space:

```rust
let mut logfile_config = sensorlog::logfile_config::LogfileConfig::new();
logfile_config.set_default_storage_quota(sensorlog::quota::StorageQuota::Unlimited);

let datadir = PathBuf::from("/tmp/sensordata");
let service = sensorlog::Sensorlog::new(&datadir, logfile_config)?;
```


Clock Watchdog
--------------

By default, sensorlog will store the current, absolute wall clock time with each
measurement. This can be problematic when running on an 'offline' system that does
not have access to a true clock reference like GPS or the internet.

Let's assume that we're running on such an offline system that requires the system
time to be configured by a field operator. Consider the case where the operator
incorrectly enters a system time far in the future and later, after noticing
the mistake, changes the system time to the correct value. How should a service
like sensorlog behave in this scenario?

The naive behaviour would be to simply fail open and continue to serve the existing
data on record. Of course, this would mean serving incorrect data without any
way for the user to tell that it is incorrect. Clearly not a good solution!

So instead, sensorlog implements a clock watchdog option that allows you to fail closed
whenever the system time changes in unexpected ways. With the clock watchdog enabled
sensorlog will watch the system clock and trigger the watchdog once it detects a
large jump in time.

The watchdog can run in either of two modes called 'panic' and 'wipe'. In the 'panic'
mode, sensorlogd will simply exit with an error message when the watchdog is triggered.
In the 'wipe' mode, triggering the watchdog will result in all stored measurement data
to be deleted.

:warning: CURRENTLY THE WATCHDOG ALWAYS RUNS IN "WIPE" MODE!!!

Design Goals
------------

Since sensorlog is designed for semi-embedded use cases, i.e. to run on a constrained
system in a high-reliability environment, the primary design goals are simplicity,
robustness and bounded resource usage.

Whenever there is a conflict between these goals and other competing goals we have
to make a tradeoff. Hence, sensorlog does not feature a particularly high operation
throughput, low operation latency or minimal
resource usage.


Caveats
-------

- sensorlog requires the time field of consecutive measurements with the same
  sensor_id to be monotonically increasing. If you try to insert a measurement that
  is older than another measurement with the same sensor_id that is already stored,
  the existing data for the sensor will be flushed.

- The specified storage quotas are applied to the total used storage space including
  sensorlog's metadata, but excluding filesystem overheads. This means the amount
  of storage actually available for measurement payload data is a bit less than the
  configured quota. Also, the amount of actual disk space used is a bit more
  than the configured quota once filesystem overheads are accounted for.

- The clock watchdog may fail to trigger in an A-B-A scenario where the system time
  changes very quickly. However, this is not a problem in practice since the only case
  in which the watchdog fails to trigger is the case in which no measurements
  were stored or retrieved during the intermittent clock change.

- While the sensorlog object should be multithread-safe, the storage is not. Do
  not create multiple instances of sensorlog that reference the same data directory
  at the same time. While this should not corrupt the data files, it can lead to
  data loss.

Alternatives Considered
-----------------------

- Another alternative strategy for dealing with clock changes would be to handle them
  gracefully by re-writing the existing data to correct for the time offset. However,
  this solution was not chosen for sensorlog since it was deemed to be a bit too expensive
  and error prone to implement in practice.


License
-------

    Copyright © 2018 nyantec GmbH <oss@nyantec.com>

    Authors:
      Paul Asmuth <asm@nyantec.com>
      Karl Engelhardt <ken@nyantec.com>

    Provided that these terms and disclaimer and all copyright notices
    are retained or reproduced in an accompanying document, permission
    is granted to deal in this work without restriction, including un‐
    limited rights to use, publicly perform, distribute, sell, modify,
    merge, give away, or sublicence.

    This work is provided “AS IS” and WITHOUT WARRANTY of any kind, to
    the utmost extent permitted by applicable law, neither express nor
    implied; without malicious intent or gross negligence. In no event
    may a licensor, author or contributor be held liable for indirect,
    direct, other damage, loss, or other issues arising in any way out
    of dealing in the work, even if advised of the possibility of such
    damage or existence of a defect, except proven that it results out
    of said person’s immediate fault when using the work as intended.
