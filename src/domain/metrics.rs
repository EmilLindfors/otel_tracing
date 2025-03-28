use std::fmt;

const NS: &str = "ns";

/// Main enum representing all possible Datadog metric unit categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricUnit {
    Bytes(BytesUnit),
    Time(TimeUnit),
    Percentage(PercentageUnit),
    Network(NetworkUnit),
    System(SystemUnit),
    Disk(DiskUnit),
    General(GeneralUnit),
    Database(DatabaseUnit),
    Cache(CacheUnit),
    Money(MoneyUnit),
    Memory(MemoryUnit),
    Frequency(FrequencyUnit),
    Logging(LoggingUnit),
    Temperature(TemperatureUnit),
    Cpu(CpuUnit),
    Power(PowerUnit),
    Current(CurrentUnit),
    Potential(PotentialUnit),
    Apm(ApmUnit),
    Synthetics(SyntheticsUnit),
    Count,
}

impl MetricUnit {
    /// Get the string representation of the unit for DataDog API
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bytes(unit) => unit.as_str(),
            Self::Time(unit) => unit.as_str(),
            Self::Percentage(unit) => unit.as_str(),
            Self::Network(unit) => unit.as_str(),
            Self::System(unit) => unit.as_str(),
            Self::Disk(unit) => unit.as_str(),
            Self::General(unit) => unit.as_str(),
            Self::Database(unit) => unit.as_str(),
            Self::Cache(unit) => unit.as_str(),
            Self::Money(unit) => unit.as_str(),
            Self::Memory(unit) => unit.as_str(),
            Self::Frequency(unit) => unit.as_str(),
            Self::Logging(unit) => unit.as_str(),
            Self::Temperature(unit) => unit.as_str(),
            Self::Cpu(unit) => unit.as_str(),
            Self::Power(unit) => unit.as_str(),
            Self::Current(unit) => unit.as_str(),
            Self::Potential(unit) => unit.as_str(),
            Self::Apm(unit) => unit.as_str(),
            Self::Synthetics(unit) => unit.as_str(),
            Self::Count => "count",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            // bytes units
            "bit" => Self::Bytes(BytesUnit::Bit),
            "byte" => Self::Bytes(BytesUnit::Byte),
            "kilobyte" => Self::Bytes(BytesUnit::KiloByte),
            "megabyte" => Self::Bytes(BytesUnit::MegaByte),
            "gigabyte" => Self::Bytes(BytesUnit::GigaByte),
            "terabyte" => Self::Bytes(BytesUnit::TeraByte),
            // Time units
            "ns" => Self::Time(TimeUnit::Nanosecond),
            "μs" => Self::Time(TimeUnit::Microsecond),
            "ms" => Self::Time(TimeUnit::Millisecond),
            "s" => Self::Time(TimeUnit::Second),
            "min" => Self::Time(TimeUnit::Minute),
            "hr" => Self::Time(TimeUnit::Hour),
            "day" => Self::Time(TimeUnit::Day),
            "wk" => Self::Time(TimeUnit::Week),
            // Percentage units
            "n%" => Self::Percentage(PercentageUnit::PercentNano),
            "%" => Self::Percentage(PercentageUnit::Percent),
            "apdex" => Self::Percentage(PercentageUnit::Apdex),
            "fraction" => Self::Percentage(PercentageUnit::Fraction),
            // Network units
            "conn" => Self::Network(NetworkUnit::Connection),
            "req" => Self::Network(NetworkUnit::Request),
            "pkt" => Self::Network(NetworkUnit::Packet),
            "seg" => Self::Network(NetworkUnit::Segment),
            "rsp" => Self::Network(NetworkUnit::Response),
            "msg" => Self::Network(NetworkUnit::Message),
            "payload" => Self::Network(NetworkUnit::Payload),
            "timeout" => Self::Network(NetworkUnit::Timeout),
            "datagram" => Self::Network(NetworkUnit::Datagram),
            "route" => Self::Network(NetworkUnit::Route),
            "session" => Self::Network(NetworkUnit::Session),
            "hop" => Self::Network(NetworkUnit::Hop),
            // System units
            "proc" => Self::System(SystemUnit::Process),
            "thread" => Self::System(SystemUnit::Thread),
            "host" => Self::System(SystemUnit::Host),
            "node" => Self::System(SystemUnit::Node),
            "fault" => Self::System(SystemUnit::Fault),
            "svc" => Self::System(SystemUnit::Service),
            "instance" => Self::System(SystemUnit::Instance),
            "cpu" => Self::System(SystemUnit::Cpu),
            // General units
            "buffer" => Self::General(GeneralUnit::Buffer),
            "err" => Self::General(GeneralUnit::Error),
            "rd" => Self::General(GeneralUnit::Read),
            "wr" => Self::General(GeneralUnit::Write),
            "occurrence" => Self::General(GeneralUnit::Occurrence),
            "event" => Self::General(GeneralUnit::Event),
            "time" => Self::General(GeneralUnit::Time),
            "unit" => Self::General(GeneralUnit::Unit),
            "op" => Self::General(GeneralUnit::Operation),
            "item" => Self::General(GeneralUnit::Item),
            "task" => Self::General(GeneralUnit::Task),
            "worker" => Self::General(GeneralUnit::Worker),
            "res" => Self::General(GeneralUnit::Resource),
            "gc" => Self::General(GeneralUnit::GarbageCollection),
            "email" => Self::General(GeneralUnit::Email),
            "smpl" => Self::General(GeneralUnit::Sample),
            "stage" => Self::General(GeneralUnit::Stage),
            "monitor" => Self::General(GeneralUnit::Monitor),
            "location" => Self::General(GeneralUnit::Location),
            "check" => Self::General(GeneralUnit::Check),
            "attempt" => Self::General(GeneralUnit::Attempt),
            "dev" => Self::General(GeneralUnit::Device),
            "up" => Self::General(GeneralUnit::Update),
            "mthd" => Self::General(GeneralUnit::Method),
            "job" => Self::General(GeneralUnit::Job),
            "container" => Self::General(GeneralUnit::Container),
            "execution" => Self::General(GeneralUnit::Execution),
            "throttle" => Self::General(GeneralUnit::Throttle),
            "invocation" => Self::General(GeneralUnit::Invocation),
            "user" => Self::General(GeneralUnit::User),
            "success" => Self::General(GeneralUnit::Success),
            "build" => Self::General(GeneralUnit::Build),
            "prediction" => Self::General(GeneralUnit::Prediction),
            "exception" => Self::General(GeneralUnit::Exception),
            // Database units
            "table" => Self::Database(DatabaseUnit::Table),
            "idx" => Self::Database(DatabaseUnit::Index),
            "lock" => Self::Database(DatabaseUnit::Lock),
            "tx" => Self::Database(DatabaseUnit::Transaction),
            "query" => Self::Database(DatabaseUnit::Query),
            "row" => Self::Database(DatabaseUnit::Row),
            "key" => Self::Database(DatabaseUnit::Key),
            "cmd" => Self::Database(DatabaseUnit::Command),
            "offset" => Self::Database(DatabaseUnit::Offset),
            "record" => Self::Database(DatabaseUnit::Record),
            "object" => Self::Database(DatabaseUnit::Object),
            "cursor" => Self::Database(DatabaseUnit::Cursor),
            "assert" => Self::Database(DatabaseUnit::Assertion),
            "scan" => Self::Database(DatabaseUnit::Scan),
            "document" => Self::Database(DatabaseUnit::Document),
            "shard" => Self::Database(DatabaseUnit::Shard),
            "flush" => Self::Database(DatabaseUnit::Flush),
            "merge" => Self::Database(DatabaseUnit::Merge),
            "refresh" => Self::Database(DatabaseUnit::Refresh),
            "fetch" => Self::Database(DatabaseUnit::Fetch),
            "col" => Self::Database(DatabaseUnit::Column),
            "commit" => Self::Database(DatabaseUnit::Commit),
            "wait" => Self::Database(DatabaseUnit::Wait),
            "ticket" => Self::Database(DatabaseUnit::Ticket),
            "question" => Self::Database(DatabaseUnit::Question),
            // Cache units
            "hit" => Self::Cache(CacheUnit::Hit),
            "miss" => Self::Cache(CacheUnit::Miss),
            "eviction" => Self::Cache(CacheUnit::Eviction),
            "get" => Self::Cache(CacheUnit::Get),
            "set" => Self::Cache(CacheUnit::Set),
            // Money units
            "$" => Self::Money(MoneyUnit::Dollar),
            "¢" => Self::Money(MoneyUnit::Cent),
            "μ$" => Self::Money(MoneyUnit::MicroDollar),
            "€" => Self::Money(MoneyUnit::Euro),
            "£" => Self::Money(MoneyUnit::Pound),
            "p" => Self::Money(MoneyUnit::Pence),
            "¥" => Self::Money(MoneyUnit::Yen),
            // Memory units
            "pg" => Self::Memory(MemoryUnit::Page),
            "split" => Self::Memory(MemoryUnit::Split),
            // Frequency units
            "Hz" => Self::Frequency(FrequencyUnit::Hertz),
            "kHz" => Self::Frequency(FrequencyUnit::Kilohertz),
            "MHz" => Self::Frequency(FrequencyUnit::Megahertz),
            "GHz" => Self::Frequency(FrequencyUnit::Gigahertz),
            // Logging units
            "entry" => Self::Logging(LoggingUnit::Entry),
            // Temperature units
            "d°C" => Self::Temperature(TemperatureUnit::DeciDegreeCelsius),
            "°C" => Self::Temperature(TemperatureUnit::DegreeCelsius),
            "°F" => Self::Temperature(TemperatureUnit::DegreeFahrenheit),
            // CPU units
            "ncores" => Self::Cpu(CpuUnit::NanoCore),
            "μcores" => Self::Cpu(CpuUnit::MicroCore),
            "mcores" => Self::Cpu(CpuUnit::MilliCore),
            "core" => Self::Cpu(CpuUnit::Core),
            "Kcores" => Self::Cpu(CpuUnit::KiloCore),
            "Mcores" => Self::Cpu(CpuUnit::MegaCore),
            "Gcores" => Self::Cpu(CpuUnit::GigaCore),
            "Tcores" => Self::Cpu(CpuUnit::TeraCore),
            "Pcores" => Self::Cpu(CpuUnit::PetaCore),
            "Ecores" => Self::Cpu(CpuUnit::ExaCore),
            // Power units
            "nW" => Self::Power(PowerUnit::Nanowatt),
            "μW" => Self::Power(PowerUnit::Microwatt),
            "mW" => Self::Power(PowerUnit::Milliwatt),
            "dW" => Self::Power(PowerUnit::Deciwatt),
            "watt" => Self::Power(PowerUnit::Watt),
            "kilowatt" => Self::Power(PowerUnit::Kilowatt),
            "megawatt" => Self::Power(PowerUnit::Megawatt),
            "gigawatt" => Self::Power(PowerUnit::Gigawatt),
            "terrawatt" => Self::Power(PowerUnit::Terrawatt),
            // Current units
            "mA" => Self::Current(CurrentUnit::Milliampere),
            "A" => Self::Current(CurrentUnit::Ampere),
            // Potential units
            "mV" => Self::Potential(PotentialUnit::Millivolt),
            "V" => Self::Potential(PotentialUnit::Volt),
            // APM units
            "span" => Self::Apm(ApmUnit::Span),
            // Synthetics units
            "run" => Self::Synthetics(SyntheticsUnit::Run),
            "step" => Self::Synthetics(SyntheticsUnit::Step),
            // Count unit
            "count" => Self::Count,
            _ => panic!("Unknown metric unit: {}", s),
        }
    }
}

impl fmt::Display for MetricUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Byte-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytesUnit {
    Bit,
    Byte,
    KiloByte,
    MegaByte,
    GigaByte,
    TeraByte,
    PetaByte,
    ExaByte,
}

impl BytesUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bit => "bit",
            Self::Byte => "byte",
            Self::KiloByte => "kilobyte",
            Self::MegaByte => "megabyte",
            Self::GigaByte => "gigabyte",
            Self::TeraByte => "terabyte",
            Self::PetaByte => "petabyte",
            Self::ExaByte => "exabyte",
        }
    }
}

/// Time-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeUnit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
}

impl TimeUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Nanosecond => "ns",
            Self::Microsecond => "μs",
            Self::Millisecond => "ms",
            Self::Second => "s",
            Self::Minute => "min",
            Self::Hour => "hr",
            Self::Day => "day",
            Self::Week => "wk",
        }
    }
}

/// Percentage-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PercentageUnit {
    PercentNano,
    Percent,
    Apdex,
    Fraction,
}

impl PercentageUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PercentNano => "n%",
            Self::Percent => "%",
            Self::Apdex => "apdex",
            Self::Fraction => "fraction",
        }
    }
}

/// Network-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkUnit {
    Connection,
    Request,
    Packet,
    Segment,
    Response,
    Message,
    Payload,
    Timeout,
    Datagram,
    Route,
    Session,
    Hop,
}

impl NetworkUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Connection => "conn",
            Self::Request => "req",
            Self::Packet => "pkt",
            Self::Segment => "seg",
            Self::Response => "rsp",
            Self::Message => "msg",
            Self::Payload => "payload",
            Self::Timeout => "timeout",
            Self::Datagram => "datagram",
            Self::Route => "route",
            Self::Session => "session",
            Self::Hop => "hop",
        }
    }
}

/// System-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemUnit {
    Process,
    Thread,
    Host,
    Node,
    Fault,
    Service,
    Instance,
    Cpu,
}

impl SystemUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Process => "proc",
            Self::Thread => "thread",
            Self::Host => "host",
            Self::Node => "node",
            Self::Fault => "fault",
            Self::Service => "svc",
            Self::Instance => "instance",
            Self::Cpu => "cpu",
        }
    }
}

/// Disk-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskUnit {
    File,
    Inode,
    Sector,
    Block,
}

impl DiskUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::File => "file",
            Self::Inode => "inode",
            Self::Sector => "sector",
            Self::Block => "blk",
        }
    }
}

/// General units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneralUnit {
    Buffer,
    Error,
    Read,
    Write,
    Occurrence,
    Event,
    Time,
    Unit,
    Operation,
    Item,
    Task,
    Worker,
    Resource,
    GarbageCollection,
    Email,
    Sample,
    Stage,
    Monitor,
    Location,
    Check,
    Attempt,
    Device,
    Update,
    Method,
    Job,
    Container,
    Execution,
    Throttle,
    Invocation,
    User,
    Success,
    Build,
    Prediction,
    Exception,
}

impl GeneralUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Buffer => "buffer",
            Self::Error => "err",
            Self::Read => "rd",
            Self::Write => "wr",
            Self::Occurrence => "occurrence",
            Self::Event => "event",
            Self::Time => "time",
            Self::Unit => "unit",
            Self::Operation => "op",
            Self::Item => "item",
            Self::Task => "task",
            Self::Worker => "worker",
            Self::Resource => "res",
            Self::GarbageCollection => "gc",
            Self::Email => "email",
            Self::Sample => "smpl",
            Self::Stage => "stage",
            Self::Monitor => "monitor",
            Self::Location => "location",
            Self::Check => "check",
            Self::Attempt => "attempt",
            Self::Device => "dev",
            Self::Update => "up",
            Self::Method => "mthd",
            Self::Job => "job",
            Self::Container => "container",
            Self::Execution => "execution",
            Self::Throttle => "throttle",
            Self::Invocation => "invocation",
            Self::User => "user",
            Self::Success => "success",
            Self::Build => "build",
            Self::Prediction => "prediction",
            Self::Exception => "exception",
        }
    }
}

/// Database-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseUnit {
    Table,
    Index,
    Lock,
    Transaction,
    Query,
    Row,
    Key,
    Command,
    Offset,
    Record,
    Object,
    Cursor,
    Assertion,
    Scan,
    Document,
    Shard,
    Flush,
    Merge,
    Refresh,
    Fetch,
    Column,
    Commit,
    Wait,
    Ticket,
    Question,
}

impl DatabaseUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Table => "table",
            Self::Index => "idx",
            Self::Lock => "lock",
            Self::Transaction => "tx",
            Self::Query => "query",
            Self::Row => "row",
            Self::Key => "key",
            Self::Command => "cmd",
            Self::Offset => "offset",
            Self::Record => "record",
            Self::Object => "object",
            Self::Cursor => "cursor",
            Self::Assertion => "assert",
            Self::Scan => "scan",
            Self::Document => "document",
            Self::Shard => "shard",
            Self::Flush => "flush",
            Self::Merge => "merge",
            Self::Refresh => "refresh",
            Self::Fetch => "fetch",
            Self::Column => "col",
            Self::Commit => "commit",
            Self::Wait => "wait",
            Self::Ticket => "ticket",
            Self::Question => "question",
        }
    }
}

/// Cache-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheUnit {
    Hit,
    Miss,
    Eviction,
    Get,
    Set,
}

impl CacheUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Eviction => "eviction",
            Self::Get => "get",
            Self::Set => "set",
        }
    }
}

/// Money-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyUnit {
    Dollar,
    Cent,
    MicroDollar,
    Euro,
    Pound,
    Pence,
    Yen,
}

impl MoneyUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Dollar => "$",
            Self::Cent => "¢",
            Self::MicroDollar => "μ$",
            Self::Euro => "€",
            Self::Pound => "£",
            Self::Pence => "p",
            Self::Yen => "¥",
        }
    }
}

/// Memory-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryUnit {
    Page,
    Split,
}

impl MemoryUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Page => "pg",
            Self::Split => "split",
        }
    }
}

/// Frequency-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrequencyUnit {
    Hertz,
    Kilohertz,
    Megahertz,
    Gigahertz,
}

impl FrequencyUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hertz => "Hz",
            Self::Kilohertz => "kHz",
            Self::Megahertz => "MHz",
            Self::Gigahertz => "GHz",
        }
    }
}

/// Logging-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoggingUnit {
    Entry,
}

impl LoggingUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Entry => "entry",
        }
    }
}

/// Temperature-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemperatureUnit {
    DeciDegreeCelsius,
    DegreeCelsius,
    DegreeFahrenheit,
}

impl TemperatureUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DeciDegreeCelsius => "d°C",
            Self::DegreeCelsius => "°C",
            Self::DegreeFahrenheit => "°F",
        }
    }
}

/// CPU-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuUnit {
    NanoCore,
    MicroCore,
    MilliCore,
    Core,
    KiloCore,
    MegaCore,
    GigaCore,
    TeraCore,
    PetaCore,
    ExaCore,
}

impl CpuUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::NanoCore => "ncores",
            Self::MicroCore => "μcores",
            Self::MilliCore => "mcores",
            Self::Core => "core",
            Self::KiloCore => "Kcores",
            Self::MegaCore => "Mcores",
            Self::GigaCore => "Gcores",
            Self::TeraCore => "Tcores",
            Self::PetaCore => "Pcores",
            Self::ExaCore => "Ecores",
        }
    }
}

/// Power-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerUnit {
    Nanowatt,
    Microwatt,
    Milliwatt,
    Deciwatt,
    Watt,
    Kilowatt,
    Megawatt,
    Gigawatt,
    Terrawatt,
}

impl PowerUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Nanowatt => "nW",
            Self::Microwatt => "μW",
            Self::Milliwatt => "mW",
            Self::Deciwatt => "dW",
            Self::Watt => "watt",
            Self::Kilowatt => "kilowatt",
            Self::Megawatt => "megawatt",
            Self::Gigawatt => "gigawatt",
            Self::Terrawatt => "terrawatt",
        }
    }
}

/// Current-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentUnit {
    Milliampere,
    Ampere,
}

impl CurrentUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Milliampere => "mA",
            Self::Ampere => "A",
        }
    }
}

/// Potential-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PotentialUnit {
    Millivolt,
    Volt,
}

impl PotentialUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Millivolt => "mV",
            Self::Volt => "V",
        }
    }
}

/// APM-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApmUnit {
    Span,
}

impl ApmUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Span => "span",
        }
    }
}

/// Synthetics-related units
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticsUnit {
    Run,
    Step,
}

impl SyntheticsUnit {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Run => "run",
            Self::Step => "step",
        }
    }
}

/// Helper functions to create common metric units
pub mod units {
    use super::*;

    // Byte-related helper functions
    pub fn bytes() -> MetricUnit {
        MetricUnit::Bytes(BytesUnit::Byte)
    }
    pub fn kib() -> MetricUnit {
        MetricUnit::Bytes(BytesUnit::KiloByte)
    }
    pub fn mib() -> MetricUnit {
        MetricUnit::Bytes(BytesUnit::MegaByte)
    }
    pub fn gib() -> MetricUnit {
        MetricUnit::Bytes(BytesUnit::GigaByte)
    }

    // Time-related helper functions
    pub fn nanoseconds() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Nanosecond)
    }
    pub fn microseconds() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Microsecond)
    }
    pub fn milliseconds() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Millisecond)
    }
    pub fn seconds() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Second)
    }
    pub fn minutes() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Minute)
    }
    pub fn hours() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Hour)
    }
    pub fn days() -> MetricUnit {
        MetricUnit::Time(TimeUnit::Day)
    }

    // Percentage-related helper functions
    pub fn percent() -> MetricUnit {
        MetricUnit::Percentage(PercentageUnit::Percent)
    }
    pub fn fraction() -> MetricUnit {
        MetricUnit::Percentage(PercentageUnit::Fraction)
    }

    // Network-related helper functions
    pub fn requests() -> MetricUnit {
        MetricUnit::Network(NetworkUnit::Request)
    }
    pub fn connections() -> MetricUnit {
        MetricUnit::Network(NetworkUnit::Connection)
    }
    pub fn packets() -> MetricUnit {
        MetricUnit::Network(NetworkUnit::Packet)
    }

    // System-related helper functions
    pub fn processes() -> MetricUnit {
        MetricUnit::System(SystemUnit::Process)
    }
    pub fn threads() -> MetricUnit {
        MetricUnit::System(SystemUnit::Thread)
    }
    pub fn hosts() -> MetricUnit {
        MetricUnit::System(SystemUnit::Host)
    }

    // General helper functions
    pub fn operations() -> MetricUnit {
        MetricUnit::General(GeneralUnit::Operation)
    }
    pub fn errors() -> MetricUnit {
        MetricUnit::General(GeneralUnit::Error)
    }
    pub fn events() -> MetricUnit {
        MetricUnit::General(GeneralUnit::Event)
    }
}
