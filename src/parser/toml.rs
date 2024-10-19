//! this module is mainly used to Deserialize Cargo.toml

use anyhow::anyhow;
use anyhow::Result;
use toml::Table;

pub fn des_toml(file: &std::path::Path,) -> Result<toml::map::Map<String, toml::Value,>,> {
	let contents = std::fs::read_to_string(file,)?;
	Ok(contents.parse::<Table>()?,)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_check() -> Result<(),> {
		let value = "[[bin]]\nname=\"foo\"".parse::<Table>()?;
		let toml::Value::Array(a,) = value.get("bin",).unwrap() else {
			return Err(anyhow!("🫠"),);
		};
		assert_eq!(a[0]["name"], toml::Value::String("foo".to_string()));
		Ok((),)
	}
}
