// Copyright (c) 2023 Jonathan "Razordor" Alan Thomason
use proc_macro2::TokenStream as TokenStream2;
use std::str::FromStr;
use syn::punctuated::Punctuated;
use syn::{spanned::Spanned, *};

pub struct AttrData {
	pub strip: bool,
	pub link_ty: LinkType,
	pub linker: Option<Ident>,
}

#[derive(PartialEq)]
pub enum LinkType {
	Vulkan,
	// note: dylink_macro must use an owned string instead of `&'static [u8]` since it's reading from the source code.
	General(Vec<String>),
}

impl TryFrom<Punctuated<Expr, Token!(,)>> for AttrData {
	type Error = syn::Error;
	fn try_from(value: Punctuated<Expr, Token!(,)>) -> Result<Self> {
		let mut maybe_strip: Option<bool> = None;
		let mut maybe_link_ty: Option<LinkType> = None;
		let mut linker: Option<Ident> = None;
		let mut errors = vec![];
		const EXPECTED_KW: &str = "Expected `vulkan`, `any`, `strip`, or `name`.";

		for expr in value.iter() {
			match expr {
				// Branch for syntax: #[dylink(vulkan)]
				Expr::Path(ExprPath { path, .. }) => {
					if path.is_ident("vulkan") {
						if maybe_link_ty.is_none() {
							maybe_link_ty = Some(LinkType::Vulkan);
						} else {
							errors.push(Error::new(path.span(), "Linkage already defined."));
						}
					} else {
						errors.push(Error::new(path.span(), EXPECTED_KW));
					}
				}
				Expr::Assign(assign) => {
					let (assign_left, assign_right) = (assign.left.as_ref(), assign.right.as_ref());

					let Expr::Path(ExprPath { path, .. }) = assign_left else {
						unreachable!("internal error when parsing Expr::Assign");
					};
					if path.is_ident("name") {
						// Branch for syntax: #[dylink(name = <string literal>)]
						match assign_right {
							Expr::Lit(ExprLit {
								lit: Lit::Str(lib), ..
							}) => {
								if maybe_link_ty.is_none() {
									maybe_link_ty = Some(LinkType::General(vec![lib.value()]))
								} else {
									errors.push(Error::new(
                                        assign.span(),
                                        "Linkage already defined. Suggested: use `any()` for checking multiple libraries.",
                                    ));
								}
							}
							right => {
								errors.push(Error::new(right.span(), "Expected string literal."))
							}
						}
					} else if path.is_ident("strip") {
						// Branch for syntax: #[dylink(strip = <bool>)]
						match assign_right {
							Expr::Lit(ExprLit {
								lit: Lit::Bool(val),
								..
							}) => {
								if maybe_strip.is_none() {
									maybe_strip = Some(val.value());
								} else {
									errors.push(Error::new(
										assign.span(),
										"strip is already defined",
									));
								}
							}
							right => {
								errors.push(Error::new(right.span(), "Expected boolean literal."))
							}
						}
					} else if path.is_ident("linker") {
						// Branch for syntax: #[dylink(linker = <ident>)]
						match assign_right {
							Expr::Path(ExprPath { path, .. }) => {
								if linker.is_none() {
									linker = Some(path.get_ident().unwrap().clone());
								} else {
									errors.push(Error::new(
										assign.span(),
										"linker is already defined",
									));
								}
							}
							right => {
								errors.push(Error::new(right.span(), "Expected identifier."))
							}
						}
					} else {
						errors.push(Error::new(assign_left.span(), EXPECTED_KW));
					}
				}
				// Branch for syntax: #[dylink(any())]
				Expr::Call(call) => {
					let call_fn = call.func.as_ref();
					if !matches!(call_fn, Expr::Path(ExprPath { path, .. }) if path.is_ident("any"))
					{
						errors.push(Error::new(call_fn.span(), "Expected function `any`."));
					} else {
						let mut lib_list = vec![];
						// This is non-recursive by design.
						// The `any` function should only be used once and vulkan style loading is no longer an option by this point.
						for arg in call.args.iter() {
							match arg {
								Expr::Assign(assign) => {
									if !matches!(assign.left.as_ref(), Expr::Path(ExprPath { path, .. }) if path.is_ident("name"))
									{
										errors.push(Error::new(
											assign.left.span(),
											"expected identifier `name`.",
										));
									}
									match assign.right.as_ref() {
										Expr::Lit(ExprLit {
											lit: Lit::Str(lib), ..
										}) => lib_list.push(lib.value()),
										right => errors.push(Error::new(
											right.span(),
											"Expected string literal.",
										)),
									}
								}
								other => errors
									.push(Error::new(other.span(), "Expected `name = <string>`.")),
							}
						}
						if lib_list.is_empty() {
							errors.push(Error::new(call.span(), "No arguments detected."));
						} else {
							maybe_link_ty = Some(LinkType::General(lib_list));
						}
					}
				}
				// Branch for everything else.
				expr => errors.push(Error::new(expr.span(), EXPECTED_KW)),
			}
		}

		if maybe_link_ty.is_none() {
			errors.push(Error::new(
				value.span(),
				"No linkage detected. Suggested: use `vulkan` or `name = <string>` for linkage.",
			));
		}

		// if there are any errors this will immediately combine and return early.
		if !errors.is_empty() {
			if let Some(mut main_err) = errors.pop() {
				for err in errors {
					main_err.combine(err);
				}
				Err(main_err)
			} else {
				// argument list was empty. this is a problem
				Err(Error::new(value.span(), EXPECTED_KW))
			}
		} else {
			Ok(Self {
				strip: maybe_strip.unwrap_or_default(),
				link_ty: maybe_link_ty.unwrap(),
				linker,
			})
		}
	}
}

impl quote::ToTokens for LinkType {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			LinkType::Vulkan => tokens
				.extend(unsafe { TokenStream2::from_str("LinkType::Vulkan").unwrap_unchecked() }),
			LinkType::General(lib_list) => {
				let mut lib_array = String::from("&unsafe {{[");
				for name in lib_list {
					lib_array.push_str(&format!(
						"std::ffi::CStr::from_bytes_with_nul_unchecked(b\"{name}\\0\"),"
					))
				}
				lib_array.push_str("]}}");
				tokens.extend(
					TokenStream2::from_str(&format!("LinkType::General({lib_array})")).unwrap(),
				)
			}
		}
	}
}
