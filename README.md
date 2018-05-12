esensord
========

[![Build Status](https://travis-ci.org/nyantec/esensord.svg?branch=master)](https://travis-ci.org/nyantec/esensord)

esensord is a lightweight data logging service.

The esensord binary is a standalone server program that allows you to store and
retrieve measurement data. Clients interact with the server via a HTTP+JSON API.
The interface is very simple and consists of only three operations:

  - `insertMeasurement(sensor_id, time, data)` - Store a measurement
  - `fetchMeasurements(sensor_id, from, until)` - Retrieve all measurements in the specified time range
  - `fetchLastMeasurement(sensor_id)` - Retrieve the most recent measurement

A measurement is a 3-tuple of `time`, `sensor_id` and `data`. The `time` field
should contain the wall clock time at which the measurement was taken. The `sensor_id`
field is used to identify a logical stream of consecutive measurements. The `data`
field is treated as an opaque blob; the contents are entirely user-defined.


Installation
------------

The fastest way to install `esensord` is via the the Cargo utility:

    $ cargo install esensord


Getting Started
---------------

Execute the following command to start esensord on HTTP port 8080. The messages
will be stored in `/var/sensordata`:

    $ mkdir /var/sensordata
    $ esensord --datadir /var/sensordata --listen_http localhost:8080 --quota_default infinite

In a shell, run this command to insert the measurement "3250" for sensor
's1.hydraulic_pressure_psi' (the time parameter will be defaulted to the current
wall clock time):

    $ curl \
        -X POST \
        -d '{ "sensor_id": "s1.hydraulic_pressure_psi", "data": "3250" }' \
        localhost:8080/api/v1/store_measurement

Afterwards, run this command to retrieve the last 10 minutes of measurements from
the 's1.hydraulic_pressure_psi' sensor:

    $ curl \
        -X POST \
        -d '{ "sensor_id": "s1.hydraulic_pressure_psi", "from": "-10min", "until": "now" }' \
        localhost:8080/api/v1/fetch_measurements

The output should look similar to this:

    < HTTP/1.1 200 OK
    < access-control-allow-origin: *
    < content-type: application/json; charset=utf-8
    < Content-Length: 944

    [
      {
        "time": 1526136156,
        "data": "3250"
      },
      {
        "time": 1526136157,
        "data": "3250"
      },
      ...
    ]


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
metadata (but excluding filesystem overheads). In other words, the amount of
storage actually available for measurement payload data is bit less than the
configured quota.


Monotonic Time
--------------

Note that esensord requires that the time of measurements is monotonically increasing.
If you try to insert a measurement that is older than another measurement that
is already stored, you will get an error message.

If you're using wall clock timestamps (which is the default), you should give some
considerations on how you want to handle large clock changes. One option for dealing
with such a scenario is to enable the clock watchdog (see below).


Clock Watchdog
--------------

By default, esensord will store the current, absolute wall clock time with each
measurement. This can be problematic when running on an 'offline' system that does
not have access to a true clock reference like GPS or the internet.

Let's assume that we're running on such an offline system that requires the system
time to be configured by a field operator. Consider the case where the operator
incorrectly enters a system time far in the future and later, after noticing
the mistake, changes the system time to the correct value.

How should a service like esensord behave in this scenario? Simply continuing to
serve the existing data with the incorrect timestamps on record is clearly not
a good solution.

While one could come up with many strategies for re-writing the existing data to
correct for the time offset, such a solution would be a bit expensive and error prone
to implement in practice.

Instead, esensord offers a 'clock watchdog'. When you enable the clock watchdog,
esensord will watch the system clock and trigger the watchdog once it detects a
large jump in time (in either direction).

The watchdog can run in either of two modes called 'panic' and 'wipe'. In the 'panic'
mode, esensord will simply exit with an error message when the watchdog is triggered.
In the 'wipe' mode, triggering the watchdog will result in all stored measurement data
to be deleted.

Here is how to enable the clock watchdog in 'panic' mode and set it up to trigger
if the time jumps forwards by more than 7 days or backwards by more than 10 minutes.

    $ esensord \
      --clock_watchdog panic \
      --clock_watchdog_trigger_forward 7days \
      --clock_watchdog_trigger_backward 10min \
      ...

Note that esensord can not differentiate between having been re-started and a forward
jump in system time. That means that shutting down the esensord service for a long period
and then re-starting could result in a false positive watchdog trigger. To prevent
this, the clock watchdog foward trigger value should bebe set to a value that is larger
than the longest expected downtime.


Configuration
-------------

The esensord distribution consists of two programs: `esensord` and `esensorctl`.
The esensord program is the main server program and the second esensorctl program
is a simple command line client.

All configuration options are set as command line arguments:

    Usage: $ esensord [OPTIONS]

    Options:

       --listen_http=<addr>
          Listen for HTTP connection on this address

       --datadir=<dir>
          Set the data directory

       --quota_default=<quota>
          Set the default storage quota for all sensors

       --quota=<sensor_id>:<quota>
           Set the storage quota for a given sensor id

       --clock_watchdog=<mode>
          Enable the clock watchdog. Modes are 'off', 'panic' and 'wipe'

       --clock_watchdog_trigger_forward=<threshold>
          Trigger the clock watchdog if the system time jumps forward by more than threshold

       --clock_watchdog_trigger_backward=<threshold>
          Trigger the clock watchdog if the system time jumps backward by more than threshold

       --daemonize
          Daemonize the server

       --pidfile=<file>
          Write a PID file

       --loglevel <level>
          Minimum log level (default: INFO)

       --[no]log_to_syslog
          Do[n't] log to syslog

       --[no]log_to_stderr
          Do[n't] log to stderr

       -?, --help
          Display this help text and exit

       -V, --version
          Display the version of this binary and exit

    Examples:
       $ esensord --datadir /var/sensordata --listen_http localhost:8080 --quota_default infinite



HTTP API
--------

The HTTP+JSON API is very simple. Below is a list of all API methods. For more
detailed documentation on the API please [refer to the the documentation](https://esensord.org)

    POST /api/v1/store_measurement
         Store a measurement for a given sensor

    POST /api/v1/fetch_last_measurement
         Retrieve the most recent measurement from a given sensor

    POST /api/v1/fetch_measurements
         Retrieve all measurements from a given sensor in the specified time range

     GET /ping
         Responds with 'pong'


Design Goals
------------

Since esensord is designed for semi-embedded use cases, i.e. to run on a constrained
system in a high-reliability environment, the primary design goals are simplicity,
robustness and bounded resource usage. Whenever there is a conflict between these
goals and other competing goals we have to make a tradeoff. Hence, esensord does
not feature a particularly high operation throughput, low operation latency or minimal
resource usage.


License
-------

    Copyright © 2018 nyantec GmbH <oss@nyantec.com>

    Authors:
      Paul Asmuth <asm@nyantec.com>

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
