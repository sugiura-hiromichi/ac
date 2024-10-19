use anyhow::anyhow;
use anyhow::Result;

/// 指定されたパスのrustファイルのソースコードのASTを返します
///
/// # Error
///
/// `file_path`からのファイル読み込みが失敗した時とソースコードのパースが失敗した時にエラーを返します
pub fn get_rs_ast(file_path: &str,) -> Result<syn::File,> {
	let code = std::fs::read_to_string(file_path,)?;
	let ast = syn::parse_file(&code,)?;
	Ok(ast,)
}

/// `ast`を`fnc`パラメータの引数として渡し、実行します
///
/// # Return
///
/// この関数は成功した場合、`fnc`の返り値を`Ok`でラップして返します
pub fn ast_rs<T: Sized, F,>(ast: &syn::File, fnc: F,) -> Result<T,>
where F: Fn(&syn::File,) -> T + Sized {
	Ok(fnc(&ast,),)
}

/// # TODO:
///
/// - 入れ子状態のパスを解決する
pub fn get_fn(ast: &syn::File, name: &str,) -> Option<syn::ItemFn,> {
	for item in &ast.items {
		if let syn::Item::Fn(f,) = item {
			if f.sig.ident.to_string() == name.to_string() {
				return Some(f.clone(),);
			}
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;

	fn this_file_ast() -> Result<syn::File,> {
		let mut cur = std::env::current_dir()?;
		cur.push("src/parser/rust.rs",);

		Ok(get_rs_ast(cur.to_str().unwrap(),)?,)
	}

	#[test]
	fn ast_get_check() -> Result<(),> {
		this_file_ast()?;
		Ok((),)
	}

	#[test]
	fn get_fn_check() -> Result<(),> {
		let ast = this_file_ast()?;
		let ast_rs = get_fn(&ast, "ast_rs",);
		assert!(ast_rs.is_some());
		Ok((),)
	}
}
