//! Scheduler probes that keep their full, licensed output in the ruled corpus.

pub const CONCURRENCY: &[(&str, &str)] = &[
    (
        "lang/sys_semaphores_unnamed.rex",
        "r[util.syseventsem.deviation.158]",
    ),
    (
        "lang/sys_semaphores_across_activities.rex",
        "r[util.sysmutexsem.deviation.158]",
    ),
];
