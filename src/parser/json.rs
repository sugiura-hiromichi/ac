use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Deserialize,)]
pub struct NodePackage {
	name:             String,
	description:      String,
	author:           String,
	license:          String,
	pub dependencies: HashMap<String, String,>,
}
