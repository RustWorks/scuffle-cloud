//! The code generator for [`tinc`](https://crates.io/crates/tinc).
#![cfg_attr(feature = "docs", doc = "\n\nSee the [changelog][changelog] for a full release history.")]
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
//! ## Usage
//!
//! In your `build.rs`:
//!
//! ```rust,no_run
//! # #[allow(clippy::needless_doctest_main)]
//! fn main() {
//!     tinc_build::Config::prost()
//!         .compile_protos(&["proto/test.proto"], &["proto"])
//!         .unwrap();
//! }
//! ```
//!
//! Look at [`Config`] to see different options to configure the generator.
//!
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![cfg_attr(not(feature = "prost"), allow(unused_variables, dead_code))]

use anyhow::Context;
use extern_paths::ExternPaths;
mod codegen;
mod extern_paths;

#[cfg(feature = "prost")]
mod prost_explore;

mod types;

/// The mode to use for the generator, currently we only support `prost` codegen.
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Use `prost` to generate the protobuf structures
    #[cfg(feature = "prost")]
    Prost,
}

impl quote::ToTokens for Mode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            #[cfg(feature = "prost")]
            Mode::Prost => quote::quote!(prost).to_tokens(tokens),
            #[cfg(not(feature = "prost"))]
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Debug)]
struct PathConfigs {
    btree_maps: Vec<String>,
    bytes: Vec<String>,
    boxed: Vec<String>,
}

/// A config for configuring how tinc builds / generates code.
#[derive(Debug)]
pub struct Config {
    disable_tinc_include: bool,
    mode: Mode,
    paths: PathConfigs,
    extern_paths: ExternPaths,
}

impl Config {
    /// New config with prost mode.
    #[cfg(feature = "prost")]
    pub fn prost() -> Self {
        Self::new(Mode::Prost)
    }

    /// Make a new config with a given mode.
    pub fn new(mode: Mode) -> Self {
        Self {
            disable_tinc_include: false,
            mode,
            paths: PathConfigs::default(),
            extern_paths: ExternPaths::new(mode),
        }
    }

    /// Disable tinc auto-include. By default tinc will add its own
    /// annotations into the include path of protoc.
    pub fn disable_tinc_include(&mut self) -> &mut Self {
        self.disable_tinc_include = true;
        self
    }

    /// Specify a path to generate a `BTreeMap` instead of a `HashMap` for proto map.
    pub fn btree_map(&mut self, path: impl std::fmt::Display) -> &mut Self {
        self.paths.btree_maps.push(path.to_string());
        self
    }

    /// Specify a path to generate `bytes::Bytes` instead of `Vec<u8>` for proto bytes.
    pub fn bytes(&mut self, path: impl std::fmt::Display) -> &mut Self {
        self.paths.bytes.push(path.to_string());
        self
    }

    /// Specify a path to wrap around a `Box` instead of including it directly into the struct.
    pub fn boxed(&mut self, path: impl std::fmt::Display) -> &mut Self {
        self.paths.boxed.push(path.to_string());
        self
    }

    /// Compile and generate all the protos with the includes.
    pub fn compile_protos(&mut self, protos: &[&str], includes: &[&str]) -> anyhow::Result<()> {
        match self.mode {
            #[cfg(feature = "prost")]
            Mode::Prost => self.compile_protos_prost(protos, includes),
        }
    }

    #[cfg(feature = "prost")]
    fn compile_protos_prost(&mut self, protos: &[&str], includes: &[&str]) -> anyhow::Result<()> {
        use codegen::prost_sanatize::to_snake;
        use codegen::utils::get_common_import_path;
        use prost_reflect::DescriptorPool;
        use quote::{ToTokens, quote};
        use syn::parse_quote;
        use types::ProtoTypeRegistry;

        let out_dir_str = std::env::var("OUT_DIR").context("OUT_DIR must be set, typically set by a cargo build script")?;
        let out_dir = std::path::PathBuf::from(&out_dir_str);
        let ft_path = out_dir.join("tinc.fd.bin");

        let mut config = prost_build::Config::new();
        config.file_descriptor_set_path(&ft_path);

        config.btree_map(self.paths.btree_maps.iter());
        self.paths.boxed.iter().for_each(|path| {
            config.boxed(path);
        });
        config.bytes(self.paths.bytes.iter());

        let mut includes = includes.to_vec();

        {
            let tinc_out = out_dir.join("tinc");
            std::fs::create_dir_all(&tinc_out).context("failed to create tinc directory")?;
            std::fs::write(tinc_out.join("annotations.proto"), tinc_pb_prost::TINC_ANNOTATIONS)
                .context("failed to write tinc_annotations.rs")?;
            includes.push(&out_dir_str);
            config.protoc_arg(format!("--descriptor_set_in={}", tinc_pb_prost::TINC_ANNOTATIONS_PB_PATH));
        }

        let fds = config.load_fds(protos, &includes).context("failed to generate tonic fds")?;

        let fds_bytes = std::fs::read(ft_path).context("failed to read tonic fds")?;

        let pool = DescriptorPool::decode(&mut fds_bytes.as_slice()).context("failed to decode tonic fds")?;

        let mut registry = ProtoTypeRegistry::new(self.mode, self.extern_paths.clone());

        config.compile_well_known_types();
        for (proto, rust) in self.extern_paths.paths() {
            let proto = if proto.starts_with('.') {
                proto.to_string()
            } else {
                format!(".{proto}")
            };
            config.extern_path(proto, rust.to_token_stream().to_string());
        }

        prost_explore::Extensions::new(&pool)
            .process(&mut registry)
            .context("failed to process extensions")?;

        let mut packages = codegen::generate_modules(&registry)?;

        packages.iter_mut().for_each(|(path, package)| {
            if self.extern_paths.contains(path) {
                return;
            }

            package.enum_configs().for_each(|(path, enum_config)| {
                if self.extern_paths.contains(path) {
                    return;
                }

                enum_config.attributes().for_each(|attribute| {
                    config.enum_attribute(path, attribute.to_token_stream().to_string());
                });
                enum_config.variants().for_each(|variant| {
                    let path = format!("{path}.{variant}");
                    enum_config.variant_attributes(variant).for_each(|attribute| {
                        config.field_attribute(&path, attribute.to_token_stream().to_string());
                    });
                });
            });

            package.message_configs().for_each(|(path, message_config)| {
                if self.extern_paths.contains(path) {
                    return;
                }

                message_config.attributes().for_each(|attribute| {
                    config.message_attribute(path, attribute.to_token_stream().to_string());
                });
                message_config.fields().for_each(|field| {
                    let path = format!("{path}.{field}");
                    message_config.field_attributes(field).for_each(|attribute| {
                        config.field_attribute(&path, attribute.to_token_stream().to_string());
                    });
                });
                message_config.oneof_configs().for_each(|(field, oneof_config)| {
                    let path = format!("{path}.{field}");
                    oneof_config.attributes().for_each(|attribute| {
                        // In prost oneofs (container) are treated as enums
                        config.enum_attribute(&path, attribute.to_token_stream().to_string());
                    });
                    oneof_config.fields().for_each(|field| {
                        let path = format!("{path}.{field}");
                        oneof_config.field_attributes(field).for_each(|attribute| {
                            config.field_attribute(&path, attribute.to_token_stream().to_string());
                        });
                    });
                });
            });

            package.extra_items.extend(package.services.iter().flat_map(|service| {
                let mut builder = tonic_build::CodeGenBuilder::new();

                builder.emit_package(true).build_transport(true);

                let make_service = |is_client: bool| {
                    let mut builder = tonic_build::manual::Service::builder()
                        .name(service.name())
                        .package(&service.package);

                    if !service.comments.is_empty() {
                        builder = builder.comment(service.comments.to_string());
                    }

                    service
                        .methods
                        .iter()
                        .fold(builder, |service_builder, (name, method)| {
                            let codec_path = if is_client {
                                quote!(::tinc::reexports::tonic::codec::ProstCodec)
                            } else {
                                let path = get_common_import_path(&service.full_name, &method.codec_path);
                                quote!(#path::<::tinc::reexports::tonic::codec::ProstCodec<_, _>>)
                            };

                            let mut builder = tonic_build::manual::Method::builder()
                                .input_type(
                                    registry
                                        .resolve_rust_path(&service.full_name, method.input.value_type().proto_path())
                                        .unwrap()
                                        .to_token_stream()
                                        .to_string(),
                                )
                                .output_type(
                                    registry
                                        .resolve_rust_path(&service.full_name, method.output.value_type().proto_path())
                                        .unwrap()
                                        .to_token_stream()
                                        .to_string(),
                                )
                                .codec_path(codec_path.to_string())
                                .name(to_snake(name))
                                .route_name(name);

                            if method.input.is_stream() {
                                builder = builder.client_streaming()
                            }

                            if method.output.is_stream() {
                                builder = builder.server_streaming();
                            }

                            if !method.comments.is_empty() {
                                builder = builder.comment(method.comments.to_string());
                            }

                            service_builder.method(builder.build())
                        })
                        .build()
                };

                let mut client: syn::ItemMod = syn::parse2(builder.generate_client(&make_service(true), "")).unwrap();
                client.content.as_mut().unwrap().1.insert(
                    0,
                    parse_quote!(
                        use ::tinc::reexports::tonic;
                    ),
                );

                let mut server: syn::ItemMod = syn::parse2(builder.generate_server(&make_service(false), "")).unwrap();
                server.content.as_mut().unwrap().1.insert(
                    0,
                    parse_quote!(
                        use ::tinc::reexports::tonic;
                    ),
                );

                [client.into(), server.into()]
            }));
        });

        config.compile_fds(fds).context("prost compile")?;

        for (package, module) in packages {
            if self.extern_paths.contains(&package) {
                continue;
            };

            let path = out_dir.join(format!("{package}.rs"));
            write_module(&path, module.extra_items).with_context(|| package.to_owned())?;
        }

        Ok(())
    }
}

fn write_module(path: &std::path::Path, module: Vec<syn::Item>) -> anyhow::Result<()> {
    let file = std::fs::read_to_string(path).context("read")?;
    let mut file = syn::parse_file(&file).context("parse")?;

    file.items.extend(module);
    std::fs::write(path, prettyplease::unparse(&file)).context("write")?;

    Ok(())
}

/// Changelogs generated by [scuffle_changelog]
#[cfg(feature = "docs")]
#[scuffle_changelog::changelog]
pub mod changelog {}
