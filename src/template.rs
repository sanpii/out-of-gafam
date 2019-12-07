use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Template {
    tera: Arc<RwLock<tera::Tera>>,
}

static PATH: &str = "templates";

impl Template
{
    pub fn new() -> Self
    {
        let path = format!("{}/**/*", PATH);
        let tera = match tera::Tera::new(&path) {
            Ok(tera) => tera,
            Err(err) => panic!("Parsing error(s): {}", err),
        };

        Self {
            tera: Arc::new(RwLock::new(tera)),
        }
    }

    pub fn render(&self, template: &str, context: &tera::Context) -> tera::Result<String>
    {
        let tera = self.tera.read()
            .unwrap();

        tera.render(template, context)
    }

    #[cfg(debug_assertions)]
    pub fn watch(self)
    {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            use notify::Watcher;

            let timeout = std::time::Duration::from_secs(2);
            let mut watcher = notify::watcher(tx, timeout)
                .unwrap();

            log::debug!("watching {} for changes", PATH);

            watcher.watch(PATH, notify::RecursiveMode::Recursive)
                .unwrap();

            loop {
                if rx.try_recv().is_ok() {
                    log::info!("shutting down template watcher");
                    return;
                }

                match rx.recv_timeout(timeout) {
                    Ok(event) => {
                        log::info!("reloading templates: {:?}", event);

                        match self.full_reload() {
                            Ok(_) => log::info!("templates reloaded"),
                            Err(e) => log::error!("failed to reload templates: {}", e),
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(e) => log::warn!("watch error: {:?}", e),
                }
            }
        });
    }

    #[cfg(debug_assertions)]
    fn full_reload(&self) -> tera::Result<()>
    {
        let mut tera = self.tera.write()
            .unwrap();

        tera.full_reload()
    }

    #[cfg(not(debug_assertions))]
    pub fn watch(self)
    {
    }
}
