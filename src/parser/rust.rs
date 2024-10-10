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

/// 指定されたパスのrustファイルのソースコードを読み込み、ASTを生成します
/// 生成されたASTを`fnc`パラメータの引数として渡し、実行します
///
/// # Return
///
/// この関数は成功した場合、`fnc`の返り値を`Ok`でラップして返します
pub fn ast_rs<T: Sized, F,>(file_path: &str, fnc: F,) -> Result<T,>
where F: Fn(&syn::File,) -> T + Sized {
	let ast = get_rs_ast(file_path,)?;
	Ok(fnc(&ast,),)
}

pub fn get_fn(ast: &syn::File,) {}
