DESCRIPTION = "sensorlogd"
LICENSE = "MirOS"
LIC_FILES_CHKSUM = "file://LICENSE;md5=b3f63d07aed9566f78411d90699e722e"

DEPENDS = ""
RDEPENDS_${PN} = ""

inherit cargo

SRC_URI += " \
	crate://crates.io/aho-corasick/0.6.4 \
	crate://crates.io/arrayvec/0.4.7 \
	crate://crates.io/atty/0.2.10 \
	crate://crates.io/base64/0.9.1 \
	crate://crates.io/bitflags/1.0.3 \
	crate://crates.io/byteorder/1.2.3 \
	crate://crates.io/bytes/0.4.7 \
	crate://crates.io/cfg-if/0.1.3 \
	crate://crates.io/crossbeam-deque/0.3.1 \
	crate://crates.io/crossbeam-epoch/0.4.1 \
	crate://crates.io/crossbeam-utils/0.3.2 \
	crate://crates.io/dtoa/0.4.2 \
	crate://crates.io/env_logger/0.5.10 \
	crate://crates.io/fuchsia-zircon-sys/0.3.3 \
	crate://crates.io/fuchsia-zircon/0.3.3 \
	crate://crates.io/futures-cpupool/0.1.8 \
	crate://crates.io/futures/0.1.21 \
	crate://crates.io/getopts/0.2.17 \
	crate://crates.io/httparse/1.2.4 \
	crate://crates.io/humantime/1.1.1 \
	crate://crates.io/hyper/0.11.26 \
	crate://crates.io/iovec/0.1.2 \
	crate://crates.io/itoa/0.4.1 \
	crate://crates.io/kernel32-sys/0.2.2 \
	crate://crates.io/language-tags/0.2.2 \
	crate://crates.io/lazy_static/1.0.0 \
	crate://crates.io/lazycell/0.6.0 \
	crate://crates.io/libc/0.2.40 \
	crate://crates.io/log/0.3.9 \
	crate://crates.io/log/0.4.1 \
	crate://crates.io/md5/0.3.7 \
	crate://crates.io/memchr/2.0.1 \
	crate://crates.io/memoffset/0.2.1 \
	crate://crates.io/mime/0.3.7 \
	crate://crates.io/mio/0.6.14 \
	crate://crates.io/miow/0.2.1 \
	crate://crates.io/net2/0.2.32 \
	crate://crates.io/nodrop/0.1.12 \
	crate://crates.io/num_cpus/1.8.0 \
	crate://crates.io/percent-encoding/1.0.1 \
	crate://crates.io/proc-macro2/0.3.8 \
	crate://crates.io/quick-error/1.2.1 \
	crate://crates.io/quote/0.5.2 \
	crate://crates.io/rand/0.3.22 \
	crate://crates.io/rand/0.4.2 \
	crate://crates.io/redox_syscall/0.1.37 \
	crate://crates.io/redox_termios/0.1.1 \
	crate://crates.io/regex-syntax/0.6.0 \
	crate://crates.io/regex/1.0.0 \
	crate://crates.io/relay/0.1.1 \
	crate://crates.io/safemem/0.2.0 \
	crate://crates.io/scoped-tls/0.1.2 \
	crate://crates.io/scopeguard/0.3.3 \
	crate://crates.io/serde/1.0.55 \
	crate://crates.io/serde_derive/1.0.55 \
	crate://crates.io/serde_json/1.0.17 \
	crate://crates.io/slab/0.3.0 \
	crate://crates.io/slab/0.4.0 \
	crate://crates.io/smallvec/0.2.1 \
	crate://crates.io/syn/0.13.9 \
	crate://crates.io/take/0.1.0 \
	crate://crates.io/termcolor/0.3.6 \
	crate://crates.io/termion/1.5.1 \
	crate://crates.io/thread_local/0.3.5 \
	crate://crates.io/time/0.1.40 \
	crate://crates.io/tokio-core/0.1.17 \
	crate://crates.io/tokio-executor/0.1.2 \
	crate://crates.io/tokio-fs/0.1.0 \
	crate://crates.io/tokio-io/0.1.6 \
	crate://crates.io/tokio-proto/0.1.1 \
	crate://crates.io/tokio-reactor/0.1.1 \
	crate://crates.io/tokio-service/0.1.0 \
	crate://crates.io/tokio-tcp/0.1.0 \
	crate://crates.io/tokio-threadpool/0.1.3 \
	crate://crates.io/tokio-timer/0.2.3 \
	crate://crates.io/tokio-udp/0.1.0 \
	crate://crates.io/tokio/0.1.6 \
	crate://crates.io/ucd-util/0.1.1 \
	crate://crates.io/unicase/2.1.0 \
	crate://crates.io/unicode-xid/0.1.0 \
	crate://crates.io/unreachable/1.0.0 \
	crate://crates.io/utf8-ranges/1.0.0 \
	crate://crates.io/version_check/0.1.3 \
	crate://crates.io/void/1.0.2 \
	crate://crates.io/winapi-build/0.1.1 \
	crate://crates.io/winapi-i686-pc-windows-gnu/0.4.0 \
	crate://crates.io/winapi-x86_64-pc-windows-gnu/0.4.0 \
	crate://crates.io/winapi/0.2.8 \
	crate://crates.io/winapi/0.3.4 \
	crate://crates.io/wincolor/0.1.6 \
	crate://crates.io/ws2_32-sys/0.2.1 \
	git://github.com/nyantec/sensorlog;protocol=https;name=sensorlog;destsuffix=sensorlog \
	"

SRCREV = "090170ec16837fa2bf9fa461b3329539bba4f7ba"

S="${WORKDIR}/sensorlog"

do_install () {
	install -d "${D}${bindir}"
	install -m 0755 "${S}/target/${HOST_SYS}/release/sensorlogd" "${D}${bindir}"
	install -m 0755 "${S}/target/${HOST_SYS}/release/sensorlogctl" "${D}${bindir}"
}

