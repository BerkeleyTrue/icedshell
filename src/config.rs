#[derive(Debug, Clone)]
struct MonitorId(String);

#[derive(Debug, Clone)]
struct Bar {
    output: MonitorId
}

#[derive(Debug, Clone)]
struct Config {
    bars: Vec<Bar>
}
