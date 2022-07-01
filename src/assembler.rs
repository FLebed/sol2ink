use std::{
    collections::HashSet,
    str::FromStr,
};

use crate::structures::*;
use convert_case::{
    Case,
    Casing,
};
use proc_macro2::TokenStream;
use quote::*;

/// Assembles ink! contract from the parsed contract struct and return it as a vec of Strings
pub fn assemble_contract(contract: Contract) -> TokenStream {
    let mod_name = format_ident!("{}", contract.name.to_case(Case::Snake));
    let signature = signature();
    let imports = assemble_imports(contract.imports);
    let events = assemble_events(contract.events);
    let enums = assemble_enums(contract.enums);
    let structs = assemble_structs(contract.structs);
    let storage = assemble_storage(contract.name.clone(), contract.fields);
    let constructor = assemble_constructor(contract.constructor);
    let functions = assemble_functions(contract.functions);
    let comments = assemble_contract_comments(contract.comments);

    let contract = quote! {
        #![cfg_attr(not(feature = "std"), no_std)]
        #![feature(min_specialization)]
        _blank_!();
        #signature
        #comments
        #[brush::contract]
        pub mod #mod_name {
            #imports
            #events
            #enums
            #structs
            #storage
            impl #mod_name {
                #constructor
                #functions
            }
        }
    };

    contract
}

/// Assembles ink! interface(trait) from the parsed interface struct and return it as a vec of Strings
pub fn assemble_interface(interface: Interface) -> TokenStream {
    let interface_name = TokenStream::from_str(&format!("{}", interface.name)).unwrap();
    let interface_name_ref = TokenStream::from_str(&format!("{}Ref", interface.name)).unwrap();
    let signature = signature();
    let imports = assemble_imports(interface.imports);
    let events = assemble_events(interface.events);
    let enums = assemble_enums(interface.enums);
    let structs = assemble_structs(interface.structs);
    let function_headers = assemble_function_headers(interface.function_headers);

    let interface = quote! {
        #signature
        #imports
        #events
        #enums
        #structs
        #[brush::wrapper]
        pub type #interface_name_ref = dyn #interface_name;
        _blank_!();
        #[brush::trait_definition]
        pub trait #interface_name {
            #function_headers
        }
    };

    interface
}

fn assemble_contract_comments(comments: Vec<String>) -> TokenStream {
    let mut output = TokenStream::new();

    // assemble comments
    for comment in comments.iter() {
        output.extend(quote! {
            #[doc = #comment]
        });
    }

    output
}

/// Sorts the imports inside the HashSet and return it as a Vec of Strings
fn assemble_imports(imports: HashSet<String>) -> TokenStream {
    let mut output = TokenStream::new();
    let mut output_vec = Vec::from_iter(imports);
    output_vec.sort();

    for import in output_vec {
        output.extend(TokenStream::from_str(&import).unwrap());
    }

    output.extend(quote! {
        _blank_!();
    });

    output
}

/// Assembles ink! enums from the vec of parsed Enum structs and return them as a vec of Strings
fn assemble_enums(enums: Vec<Enum>) -> TokenStream {
    let mut output = TokenStream::new();

    for enumeration in enums.iter() {
        let enum_name = TokenStream::from_str(&format!("{}", enumeration.name)).unwrap();
        let mut enum_comments = TokenStream::new();
        let mut values = TokenStream::new();

        // assemble comments
        for comment in enumeration.comments.iter() {
            enum_comments.extend(quote! {
                #[doc = #comment]
            });
        }

        // assemble enum values
        for value in enumeration.values.iter() {
            let value_name = TokenStream::from_str(value).unwrap();

            values.extend(quote! {
                #value_name,
            });
        }

        output.extend(quote! {
            #enum_comments
            pub enum #enum_name {
                #values
            }
            _blank_!();
        });
    }

    output
}

/// Assembles ink! events from the vec of parsed Event structs and return them as a vec of Strings
fn assemble_events(events: Vec<Event>) -> TokenStream {
    let mut output = TokenStream::new();

    for i in 0..events.len() {
        let event = &events[i];
        let event_name = TokenStream::from_str(&format!("{}", event.name)).unwrap();
        let mut event_comments = TokenStream::new();
        let mut event_fields = TokenStream::new();

        // assemble comments
        for comment in event.comments.iter() {
            event_comments.extend(quote! {
                #[doc = #comment]
            });
        }

        // assemble event fields
        for event_field in event.fields.iter() {
            if event_field.indexed {
                event_fields.extend(quote! {
                    #[ink(topic)]
                });
            }

            let event_field_name = format_ident!("{}", event_field.name.to_case(Case::Snake));
            let event_field_type = TokenStream::from_str(&event_field.field_type).unwrap();

            event_fields.extend(quote! {
                #event_field_name: #event_field_type,
            });
        }

        output.extend(quote! {
            #event_comments
            #[ink(event)]
            pub struct #event_name
            {
                #event_fields
            }
            _blank_!();
        });
    }

    output
}

/// Assembles ink! storage struct from the vec of parsed ContractField structs and return it as a vec of Strings
fn assemble_storage(contract_name: String, fields: Vec<ContractField>) -> TokenStream {
    let mut output = TokenStream::new();
    let contract_name = format_ident!("{}", contract_name);
    let mut storage_fields = TokenStream::new();

    // assemble storage fields
    for field in fields.iter() {
        let field_name = format_ident!("{}", field.name.to_case(Case::Snake));
        let field_type = TokenStream::from_str(&field.field_type).unwrap();
        storage_fields.extend(quote! {
            #field_name: #field_type,
        });
    }

    output.extend(quote! {
        #[ink(storage)]
        #[derive(Default, SpreadAllocate)]
        pub struct #contract_name {
            #storage_fields
        }
        _blank_!();
    });

    output
}

/// Assembles ink! structs from the vec of parsed Struct structs and return them as a vec of Strings
fn assemble_structs(structs: Vec<Struct>) -> TokenStream {
    let mut output = TokenStream::new();

    for i in 0..structs.len() {
        let structure = &structs[i];
        let struct_name = TokenStream::from_str(&structure.name).unwrap();
        let mut struct_comments = TokenStream::new();
        let mut struct_fields = TokenStream::new();

        // assemble comments
        for comment in structure.comments.iter() {
            struct_comments.extend(quote! {
                #[doc = #comment]
            });
        }

        // assemble struct fields
        for struct_field in structure.fields.iter() {
            let struct_field_name = format_ident!("{}", struct_field.name.to_case(Case::Snake));
            let struct_field_type = TokenStream::from_str(&struct_field.field_type).unwrap();

            struct_fields.extend(quote! {
                #struct_field_name: #struct_field_type,
            });
        }

        output.extend(quote! {
            #struct_comments
            #[derive(Default, Encode, Decode)]
            #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
            pub struct #struct_name {
                #struct_fields
            }
        });

        output.extend(quote! {
            _blank_!();
        });
    }

    output
}

/// Assembles ink! cosntructor from the parsed Function struct and return it as a vec of Strings
fn assemble_constructor(constructor: Function) -> TokenStream {
    let mut output = TokenStream::new();
    let mut params = TokenStream::new();

    // assemble params
    for param in constructor.header.params.iter() {
        let param_name = format_ident!("{}", param.name.to_case(Case::Snake));
        let param_type = TokenStream::from_str(&param.param_type).unwrap();

        params.extend(quote! {
            #param_name: #param_type,
        });
    }

    let mut body = TokenStream::new();

    // assemble body
    for statement in constructor.body.iter() {
        // TODO remove comments
        let content = format!("// {}", statement.content);
        body.extend(TokenStream::from_str(&content).unwrap());
    }

    output.extend(quote! {
        #[ink(constructor)]
        pub fn new(#params) -> Self{
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                #body
            })
        }
        _blank_!();
    });

    output
}

/// Assembles ink! functions from the vec of parsed Function structs and return them as a vec of Strings
fn assemble_functions(functions: Vec<Function>) -> TokenStream {
    let mut output = TokenStream::new();

    for i in 0..functions.len() {
        let function = &functions[i];
        let mut message = TokenStream::new();
        let mut function_name = TokenStream::new();
        let mut view = TokenStream::new();
        let mut params = TokenStream::new();
        let mut return_params = TokenStream::new();
        let mut body = TokenStream::new();

        // assemble message
        if function.header.external {
            if function.header.payable {
                message.extend(quote! {
                    #[ink(message, payable)]
                });
            } else {
                message.extend(quote! {
                    #[ink(message)]
                });
            }
        }

        // assemble function name
        function_name.extend(
            TokenStream::from_str(&format!(
                "{}{}",
                if function.header.external {
                    String::from("pub fn ")
                } else {
                    String::from("fn _")
                },
                function.header.name.to_case(Case::Snake)
            ))
            .unwrap(),
        );

        // assemble view
        view.extend(
            TokenStream::from_str(match function.header.view {
                true => "&self",
                false => "&mut self",
            })
            .unwrap(),
        );

        // assemble params
        for param in function.header.params.iter() {
            let param_name = format_ident!("{}", param.name.to_case(Case::Snake));
            let param_type = TokenStream::from_str(&param.param_type).unwrap();

            params.extend(quote! {
                , #param_name: #param_type
            });
        }

        // assemble return params
        if !function.header.return_params.is_empty() {
            let mut params = TokenStream::new();
            for i in 0..function.header.return_params.len() {
                let param_type =
                    TokenStream::from_str(&function.header.return_params[i].param_type).unwrap();

                if i > 0 {
                    params.extend(quote! {,});
                }
                params.extend(quote! {
                    #param_type
                });
            }

            if function.header.return_params.len() > 1 {
                return_params.extend(quote! {
                    -> (#params)
                });
            } else {
                return_params.extend(quote! {
                    -> #params
                });
            }
        }

        // body
        for statement in function.body.iter() {
            let content = &statement.content;
            if statement.comment {
                body.extend(quote! {
                    _comment_!(#content);
                });
            } else {
                body.extend(TokenStream::from_str(content).unwrap());
            }
        }

        // TODO remove todo
        output.extend(quote! {
            #message
            #function_name(#view #params) #return_params {
                #body
                todo!()
            }
        });

        if i < functions.len() - 1 {
            output.extend(quote! {
                _blank_!();
            });
        }
    }

    output
}

/// Assembles ink! trait function headers from the vec of parsed FunctionHeader structs and return them as a vec of Strings
fn assemble_function_headers(function_headers: Vec<FunctionHeader>) -> TokenStream {
    let mut output = TokenStream::new();

    for i in 0..function_headers.len() {
        let function = &function_headers[i];
        let mut function_comments = TokenStream::new();
        let mut message = TokenStream::new();
        let mut function_name = TokenStream::new();
        let mut view = TokenStream::new();
        let mut params = TokenStream::new();
        let mut return_params = TokenStream::new();

        // assemble comments
        for comment in function.comments.iter() {
            function_comments.extend(quote! {
                #[doc = #comment]
            });
        }

        // assemble message
        if function.external {
            if function.payable {
                message.extend(quote! {
                    #[ink(message, payable)]
                });
            } else {
                message.extend(quote! {
                    #[ink(message)]
                });
            }
        }

        // assemble function name
        function_name.extend(
            TokenStream::from_str(&format!("fn {}", function.name.to_case(Case::Snake))).unwrap(),
        );

        // assemble view
        view.extend(
            TokenStream::from_str(match function.view {
                true => "&self",
                false => "&mut self",
            })
            .unwrap(),
        );

        // assemble params
        for param in function.params.iter() {
            let param_name = format_ident!("{}", param.name.to_case(Case::Snake));
            let param_type = TokenStream::from_str(&param.param_type).unwrap();

            params.extend(quote! {
                , #param_name: #param_type
            });
        }

        // assemble return params
        if !function.return_params.is_empty() {
            let mut params = TokenStream::new();
            for i in 0..function.return_params.len() {
                let param_type =
                    TokenStream::from_str(&function.return_params[i].param_type).unwrap();

                if i > 0 {
                    params.extend(quote! {,});
                }
                params.extend(quote! {
                    #param_type
                });
            }

            if function.return_params.len() > 1 {
                return_params.extend(quote! {
                    -> (#params)
                });
            } else {
                return_params.extend(quote! {
                    -> #params
                });
            }
        }

        output.extend(quote! {
            #function_comments
            #message
            #function_name(#view #params) #return_params;
        });

        if i < function_headers.len() - 1 {
            output.extend(quote! {
                _blank_!();
            });
        }
    }

    output
}

/// Adds a signature to the beginning of the file :)
fn signature() -> TokenStream {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let version = &format!("Generated with Sol2Ink v{}\n", VERSION);
    let link = "https://github.com/Supercolony-net/sol2ink\n";
    quote! {
        _comment_!(#version);
        _comment_!(#link);
        _blank_!();
    }
}
