/// Embedded stdlib module source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StdlibSource {
    pub module: &'static str,
    pub source: &'static str,
}

pub const STDLIB_SOURCES: &[StdlibSource] = &[
    StdlibSource {
        module: "sifr.test",
        source: include_str!("../../../lib/sifr/test.sifr"),
    },
    StdlibSource {
        module: "sifr.env",
        source: include_str!("../../../lib/sifr/env.sifr"),
    },
    StdlibSource {
        module: "sifr.bytes",
        source: include_str!("../../../lib/sifr/bytes.sifr"),
    },
    StdlibSource {
        module: "sifr.encoding",
        source: include_str!("../../../lib/sifr/encoding.sifr"),
    },
    StdlibSource {
        module: "sifr.unicode",
        source: include_str!("../../../lib/sifr/unicode.sifr"),
    },
    StdlibSource {
        module: "sifr.i18n",
        source: include_str!("../../../lib/sifr/i18n.sifr"),
    },
    StdlibSource {
        module: "sifr.base64",
        source: include_str!("../../../lib/sifr/base64.sifr"),
    },
    StdlibSource {
        module: "sifr.math",
        source: include_str!("../../../lib/sifr/math.sifr"),
    },
    StdlibSource {
        module: "sifr.hashlib",
        source: include_str!("../../../lib/sifr/hashlib.sifr"),
    },
    StdlibSource {
        module: "sifr.io",
        source: include_str!("../../../lib/sifr/io.sifr"),
    },
    StdlibSource {
        module: "sifr.os",
        source: include_str!("../../../lib/sifr/os.sifr"),
    },
    StdlibSource {
        module: "sifr.json",
        source: include_str!("../../../lib/sifr/json.sifr"),
    },
    StdlibSource {
        module: "sifr.time",
        source: include_str!("../../../lib/sifr/time.sifr"),
    },
    StdlibSource {
        module: "sifr.random",
        source: include_str!("../../../lib/sifr/random.sifr"),
    },
    StdlibSource {
        module: "sifr.re",
        source: include_str!("../../../lib/sifr/re.sifr"),
    },
    StdlibSource {
        module: "sifr.collections",
        source: include_str!("../../../lib/sifr/collections.sifr"),
    },
    StdlibSource {
        module: "sifr.sync",
        source: include_str!("../../../lib/sifr/sync.sifr"),
    },
    StdlibSource {
        module: "sifr.threading",
        source: include_str!("../../../lib/sifr/threading.sifr"),
    },
    StdlibSource {
        module: "sifr.concurrent",
        source: include_str!("../../../lib/sifr/concurrent.sifr"),
    },
    StdlibSource {
        module: "sifr.asyncio",
        source: include_str!("../../../lib/sifr/asyncio.sifr"),
    },
    StdlibSource {
        module: "sifr.string",
        source: include_str!("../../../lib/sifr/string.sifr"),
    },
    StdlibSource {
        module: "sifr.bisect",
        source: include_str!("../../../lib/sifr/bisect.sifr"),
    },
    StdlibSource {
        module: "sifr.functools",
        source: include_str!("../../../lib/sifr/functools.sifr"),
    },
    StdlibSource {
        module: "sifr.secrets",
        source: include_str!("../../../lib/sifr/secrets.sifr"),
    },
    StdlibSource {
        module: "sifr.graphlib",
        source: include_str!("../../../lib/sifr/graphlib.sifr"),
    },
    StdlibSource {
        module: "sifr.uuid",
        source: include_str!("../../../lib/sifr/uuid.sifr"),
    },
    StdlibSource {
        module: "sifr.platform",
        source: include_str!("../../../lib/sifr/platform.sifr"),
    },
    StdlibSource {
        module: "sifr.pathlib",
        source: include_str!("../../../lib/sifr/pathlib.sifr"),
    },
    StdlibSource {
        module: "sifr.logging",
        source: include_str!("../../../lib/sifr/logging.sifr"),
    },
    StdlibSource {
        module: "sifr.heapq",
        source: include_str!("../../../lib/sifr/heapq.sifr"),
    },
    StdlibSource {
        module: "sifr.itertools",
        source: include_str!("../../../lib/sifr/itertools.sifr"),
    },
    StdlibSource {
        module: "sifr.textwrap",
        source: include_str!("../../../lib/sifr/textwrap.sifr"),
    },
    StdlibSource {
        module: "sifr.csv",
        source: include_str!("../../../lib/sifr/csv.sifr"),
    },
    StdlibSource {
        module: "sifr.argparse",
        source: include_str!("../../../lib/sifr/argparse.sifr"),
    },
    StdlibSource {
        module: "sifr.fnmatch",
        source: include_str!("../../../lib/sifr/fnmatch.sifr"),
    },
    StdlibSource {
        module: "sifr.shutil",
        source: include_str!("../../../lib/sifr/shutil.sifr"),
    },
    StdlibSource {
        module: "sifr.tempfile",
        source: include_str!("../../../lib/sifr/tempfile.sifr"),
    },
    StdlibSource {
        module: "sifr.difflib",
        source: include_str!("../../../lib/sifr/difflib.sifr"),
    },
    StdlibSource {
        module: "sifr.ipaddress",
        source: include_str!("../../../lib/sifr/ipaddress.sifr"),
    },
    StdlibSource {
        module: "sifr.timeit",
        source: include_str!("../../../lib/sifr/timeit.sifr"),
    },
    StdlibSource {
        module: "sifr.tomllib",
        source: include_str!("../../../lib/sifr/tomllib.sifr"),
    },
    StdlibSource {
        module: "sifr.datetime",
        source: include_str!("../../../lib/sifr/datetime.sifr"),
    },
    StdlibSource {
        module: "sifr.operator",
        source: include_str!("../../../lib/sifr/operator.sifr"),
    },
    StdlibSource {
        module: "sifr.calendar",
        source: include_str!("../../../lib/sifr/calendar.sifr"),
    },
    StdlibSource {
        module: "sifr.html",
        source: include_str!("../../../lib/sifr/html.sifr"),
    },
    StdlibSource {
        module: "sifr.sys",
        source: include_str!("../../../lib/sifr/sys.sifr"),
    },
    StdlibSource {
        module: "sifr.subprocess",
        source: include_str!("../../../lib/sifr/subprocess.sifr"),
    },
    StdlibSource {
        module: "sifr.gzip",
        source: include_str!("../../../lib/sifr/gzip.sifr"),
    },
    StdlibSource {
        module: "sifr.zipfile",
        source: include_str!("../../../lib/sifr/zipfile.sifr"),
    },
    StdlibSource {
        module: "sifr.configparser",
        source: include_str!("../../../lib/sifr/configparser.sifr"),
    },
    StdlibSource {
        module: "sifr.statistics",
        source: include_str!("../../../lib/sifr/statistics.sifr"),
    },
    StdlibSource {
        module: "sifr.glob",
        source: include_str!("../../../lib/sifr/glob.sifr"),
    },
];
