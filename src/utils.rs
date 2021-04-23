use crate::Result;
pub fn notify(title: &str, msg: &str) -> Result<()> {
    std::process::Command::new("notify-send")
        .args(&["-t", "10000", title, msg])
        .spawn()?;
        Ok(())
}
