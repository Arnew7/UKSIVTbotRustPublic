use super::replace::replacements_main;

pub async fn cycle_work_replace() {
    loop {
        match replacements_main().await {
            Ok(_) => {}
            Err(_) => {}
        };
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}