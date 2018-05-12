esensord
========

esensord is a lightweight data logging service.

The esensord binary is a standalone server program that allows you to store and
retrieve measurement data. Clients interact with the server via a HTTP+JSON API.
The interface is very simple and consists of only three operations:

    - `insertMeasurement(sensor_id, time, data)` - Store a measurement
    - `fetchMeasurements(sensor_id, from, until)` - Retrieves all measurements from a given sensor in the specified time range
    - `fetchLastMeasurement(sensor_id)` - Retrieve the most recent measurement from a given sensor

A measurement is a 3-tuple of `time`, `sensor_id` and `data`. The `time` field
should contain the wall clock time at which the measurement was taken. The `sensor_id`
field is used to identify a logical stream of consecutive measurements. The `data`
field is treated as an opaque blob; the contents are entirely user-defined.

Since esensord is designed for semi-embedded use cases, i.e. to run on a constrained
system in a high-reliability environment, the primary design goals are simplicity,
robustness and bounded resource usage. Whenever there is a conflict between these
goals and other competing goals we have to make a tradeoff. Hence, esensord does
not feature a particularly high operation throughput, low operation latency or minimal
resource usage.


Retention & Quotas
------------------

The retention and garbage collection system of esensord is based around a simple
storage quota that is assigned to each `sensor_id`. Quotas are expressed in bytes
and rotation of measurements is always first-in, first-out.

Quotas are configured using command line flags when starting the esensord server.
You can set a default quota value that applies to all sensors as well as a quota
override for each individual sensor id.

For example, to start esensord in a "whitelisting" configuration, set the default
quota to zero and then explicitly allocate storage for each sensor.

    $ esensord \
        --listen_http localhost:8080 \
        --quota_default zero \
        --quota my.first.key:1MB \
        --quota some/other/key:4MB

In the above configuration, the total disk space used by esensord will be bounded,
but you can not insert data from sensors that are not pre-configured. The exact
opposite configuration would be setting the default quota to infinite. This
configuration allows you to store data from not previously known sensors, but
may use an unbounded amount of disk space:

    $ esensord \
        --listen_http localhost:8080 \
        --quota_default infinite

Note that the specified quota is applied to the total used storage space including
metadata and other overheads; i.e. the amount of storable payload measurement
data is smaller than the configured quota.

