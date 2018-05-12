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

