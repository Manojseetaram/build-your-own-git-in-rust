use anyhow::Result;
use git2::Repository;

pub fn invoke(url: &str, dir: &str) -> Result<()> {
    let repo = Repository::clone(url, dir)?;
    println!("Cloned {} to {}", url, dir);
    Ok(())
}

