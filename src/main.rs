#![feature(pattern, never_type, file_buffered)]
#![allow(dead_code, unused_imports, unused_variables, unreachable_code)]
//! # TODO:
//! - [ ] AI機能の追加
//! - [ ] ドキュメンテーションの用意
//! - [ ] ↑を英語に(何かしらの翻訳サービス使いたい)
//! - [ ] コマンドの機能をライブラリとして提供する `libac`
//! - [ ] ↑その上で、lspのように`ac`にプロトコルとしての役割を持たせる。言うなればdep(stands for
//!   develping environment protocol)
//! - [ ] 履歴機能の追加

use anyhow::anyhow;
use anyhow::Result;
use execution_detail::ProjectType;
use project_manager::ProjectManager;
use std::path::PathBuf;

mod execution_detail;
mod parser;
mod project_manager;
mod util {

	#[derive(Debug,)]
	pub struct Queue<T: Clone,> {
		body: Option<Box<Node<T,>,>,>,
		head: *mut Option<Box<Node<T,>,>,>,
		/// このメンバーは、常に`Node`の`next`メンバーを指しています
		last: *mut Option<Box<Node<T,>,>,>,
	}

	impl<T: Clone,> Queue<T,> {
		/// この関数は初期化をしないので必ず使う前に`self.init()`を呼び出して初期化をしてください
		pub fn new() -> Self {
			Queue { body: None, head: std::ptr::null_mut(), last: std::ptr::null_mut(), }
		}

		pub fn init(&mut self, val: T,) {
			self.body = Some(Box::new(Node::new(val,),),);
			self.head = &mut self.body;
			self.last = &mut self.body.as_mut().unwrap().next;
		}

		/// `Queue`の最後に`val`を追加します
		pub fn enqueue(&mut self, val: T,) {
			unsafe {
				*self.last = Some(Box::new(Node::new(val,),),);
				self.last = &mut (*self.last).as_mut().unwrap().next;
			}
		}

		/// `Queue`の先頭を消費してその値を返します
		pub fn dequeue(&mut self,) -> Option<T,> {
			unsafe {
				if (*self.head).is_none() {
					None
				} else {
					// この操作後もself.lastは正しいアドレスを指すか？
					let val = (*self.head).as_ref().unwrap().val.clone();
					self.head = &mut (*self.head).as_mut().unwrap().next;
					Some(val,)
				}
			}
		}

		/// # Panic
		///
		/// `self`が何も含まない時パニックを起こします。`self.
		/// is_empty`メソッドを使って都度確認しましょう。
		pub fn peek(&self,) -> &T { unsafe { &(*self.head).as_ref().unwrap().val } }

		pub fn is_empty(&self,) -> bool { unsafe { (*self.head).is_none() } }
	}

	#[derive(Clone, Debug,)]
	struct Node<T: Clone,> {
		val:  T,
		next: Option<Box<Node<T,>,>,>,
	}

	impl<T: Clone,> Node<T,> {
		pub fn new(val: T,) -> Self { Self { val, next: None, } }
	}

	impl<T: Clone,> From<&[T],> for Node<T,> {
		fn from(value: &[T],) -> Self {
			Self {
				val:  value[0].clone(),
				next: if value.len() == 1 {
					None
				} else {
					Some(Box::new(Self::from(&value[1..],),),)
				},
			}
		}
	}
}

/// 実際のコマンドの実行とエラー処理はこの`main`関数で行われます
fn main() -> Result<(),> {
	let pm = ProjectManager::init()?;

	Err(anyhow!("this project is still imcomplete"),)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn take_option_changes_address() {
		let mut a = Some(0,);
		let a_addr_before: *const Option<i32,> = &a;
		let b = a.take();
		let b_addr: *const Option<i32,> = &b;
		let a_addr_after: *const Option<i32,> = &a;

		assert_eq!(a_addr_before, a_addr_after);
		assert_ne!(b_addr, a_addr_before);
	}

	#[test]
	#[should_panic]
	fn queue_pointer_check() {
		let mut a = util::Queue::new();
		a.init(0,);
		assert_eq!(a.dequeue(), Some(0));
		assert!(a.is_empty());

		a.enqueue(666,);
		a.enqueue(66,);
		a.enqueue(6,);

		assert!(!a.is_empty());
		assert_eq!(a.peek(), &666);

		assert_eq!(a.dequeue(), Some(666));
		assert_eq!(a.dequeue(), Some(66));
		assert_eq!(a.dequeue(), Some(6));
		assert!(a.is_empty());
		a.peek();
	}
}
