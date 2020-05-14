use crate::ast::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(input: Input<'_>) -> TokenStream {
    match input {
        Input::Struct(input) => gen_input(input),
    }
}

fn gen_input(input: Struct<'_>) -> TokenStream {
    let struct_name = &input.ident;
    let status = input.status();
    let intercom = input.intercom();
    let stop = input.stop();
    let start = input.start();
    let new = input.new();

    quote! {
        #[async_trait::async_trait]
        #[allow(clippy::unit_arg)]
        impl ::organix::Organix for #struct_name {
            #new
            #start
            #status
            #intercom
            #stop
        }
    }
}

impl<'a> Struct<'a> {
    fn fields(&self) -> impl Iterator<Item = &Field<'a>> {
        self.fields.iter().filter(|field| !field.skip())
    }

    fn possible_values(&self) -> Vec<TokenStream> {
        self.fields()
            .map(|field| {
                let field_name = field.original.ident.as_ref().unwrap();
                let entry = field_name.to_string();
                quote! {
                    #entry
                }
            })
            .collect()
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(&self) -> TokenStream {
        let default_is_shared = self.default_is_shared();
        let cases = self.fields().map(|field| {
            let field_name = field.original.ident.as_ref().unwrap();
            let thread_name = field_name.to_string();

            if field.shared(default_is_shared) {
                quote! {
                    #field_name: {
                        let rt = runtimes.shared_mut();
                        ::organix::service::ServiceManager::with_runtime(rt)
                    }
                }
            } else {
                let io_driver = field.io_driver();
                let time_driver = field.time_driver();

                quote! {
                    #field_name: {
                        let mut cfg = ::organix::runtime::RuntimeConfig::new(#thread_name);
                        cfg.io_driver = #io_driver;
                        cfg.time_driver = #time_driver;
                        let mut rt = ::organix::runtime::Runtime::build(cfg).unwrap();
                        let sm = ::organix::service::ServiceManager::with_runtime(&mut rt);
                        runtimes.add(rt);
                        sm
                    }
                }
            }
        });

        quote! {
            fn new(runtimes: &mut ::organix::runtime::Runtimes) -> Self {
                Self {
                    #( #cases ),*
                }
            }
        }
    }

    fn start(&self) -> TokenStream {
        let possible_values = self.possible_values();

        let cases = self.fields().map(|field| {
            let field_name = field.original.ident.as_ref().unwrap();
            let entry = field_name.to_string();
            quote! {
                #entry => {
                    match self.#field_name.runtime(watchdog_query) {
                        Ok(rt) => Ok(rt.start()),
                        Err(source) => Err(::organix::WatchdogError::CannotStartService {
                            service_identifier,
                            source,
                        })
                    }
                }
            }
        });

        quote! {
            fn start(
                &mut self,
                service_identifier: ::organix::ServiceIdentifier,
                watchdog_query: ::organix::WatchdogQuery,
            ) -> Result<(), ::organix::WatchdogError> {
                match service_identifier {
                    #( #cases ),*
                    _ => Err(::organix::WatchdogError::UnknownService {
                        service_identifier,
                        possible_values: &[#( #possible_values ),*],
                    })
                }
            }
        }
    }

    fn stop(&self) -> TokenStream {
        let possible_values = self.possible_values();

        let cases = self.fields().map(|field| {
            let field_name = field.original.ident.as_ref().unwrap();
            let entry = field_name.to_string();
            quote! {
                #entry => { Ok(self.#field_name.shutdown()) }
            }
        });

        quote! {
            fn stop(
                &mut self,
                service_identifier: ::organix::ServiceIdentifier,
            ) -> Result<(), ::organix::WatchdogError> {
                match service_identifier {
                    #( #cases ),*
                    _ => Err(::organix::WatchdogError::UnknownService {
                        service_identifier,
                        possible_values: &[#( #possible_values ),*],
                    })
                }
            }
        }
    }

    fn intercom(&self) -> TokenStream {
        let possible_values = self.possible_values();

        let cases = self.fields().map(|field| {
            let field_name = field.original.ident.as_ref().unwrap();
            let entry = field_name.to_string();
            quote! {
                #entry => { Ok(Box::new(self.#field_name.intercom())) }
            }
        });

        quote! {
            fn intercoms(
                &mut self,
                service_identifier: ::organix::ServiceIdentifier,
            ) -> Result<Box<dyn ::std::any::Any + Send>, ::organix::WatchdogError> {
                match service_identifier {
                    #( #cases ),*
                    _ => Err(::organix::WatchdogError::UnknownService {
                        service_identifier,
                        possible_values: &[#( #possible_values ),*],
                    })
                }
            }
        }
    }

    fn status(&self) -> TokenStream {
        let possible_values = self.possible_values();

        let cases = self.fields().map(|field| {
            let field_name = field.original.ident.as_ref().unwrap();
            let entry = field_name.to_string();
            quote! {
                #entry => { Ok(self.#field_name.status().await) }
            }
        });

        quote! {
            async fn status(
                &mut self,
                service_identifier: ::organix::ServiceIdentifier,
            ) -> Result<::organix::service::StatusReport, ::organix::WatchdogError> {
                match service_identifier {
                    #( #cases ),*
                    _ => Err(::organix::WatchdogError::UnknownService {
                        service_identifier,
                        possible_values: &[#( #possible_values ),*],
                    })
                }
            }
        }
    }
}
