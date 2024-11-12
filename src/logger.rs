
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let base_config = fern::Dispatch::new();
    
    let console_config = fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args! {
                "[{} {}] {} {}:{}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            })
        })
        .chain(std::io::stdout());
    
    let file_config = fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args! {
                "[{} {}] {} {}:{}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            })
        })
        .chain(fern::log_file("uzume_backend.log")?);

    base_config
        .chain(console_config)
        .chain(file_config)
        .apply()?;
    
    Ok(())
}
