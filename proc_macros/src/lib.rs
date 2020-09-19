// SPDX-License-Identifier: GPL-3.0-or-later

use {
    inflector::cases::pascalcase::to_pascal_case,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::quote,
    syn::{
        braced,
        parse::{Parse, ParseStream, Result},
        parse_macro_input,
        punctuated::Punctuated,
        token::Brace,
        ExprRange, Ident, Token, Type, Visibility,
    },
};

struct Register {
    visibility: Visibility,
    _struct_token: Token![struct],
    name: Ident,
    _colon_token: Token![:],
    ty: Type,
    _brace_token: Brace,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for Register {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility = input.parse()?;
        let struct_token = input.parse()?;
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;

        let content;
        let brace_token = braced!(content in input);

        Ok(Self {
            visibility,
            _struct_token: struct_token,
            name,
            _colon_token: colon_token,
            ty,
            _brace_token: brace_token,
            fields: content.parse_terminated(Field::parse)?,
        })
    }
}

struct Field {
    name: Ident,
    _colon_token: Token![:],
    range: ExprRange,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon_token: input.parse()?,
            range: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn add_register_type(stream: TokenStream) -> TokenStream {
    let Register {
        visibility,
        _struct_token: _,
        name,
        _colon_token: _,
        ty,
        _brace_token: _,
        fields,
    } = parse_macro_input!(stream as Register);

    let enum_name = Ident::new(
        &format!("{}{}", to_pascal_case(&name.to_string()), "Field"),
        Span::call_site(),
    );

    let enum_variants = fields
        .iter()
        .map(|field| Ident::new(&to_pascal_case(&field.name.to_string()), Span::call_site()))
        .collect::<Vec<_>>();

    let bit_range = fields.iter().map(|field| &field.range).collect::<Vec<_>>();

    let expanded = quote! {
        #visibility struct #name{
            base:x86_64::VirtAddr,
        }

        impl #name{
            #visibility fn new(phys_base:x86_64::PhysAddr)->Self{
                use {x86_64::structures::paging::{PhysFrame,Mapper,PageTableFlags},crate::mem::{allocator::{phys::FRAME_MANAGER,virt},paging::pml4::PML4}};

                const PANIC_MSG:&str="OOM during creating a new instance of register type.";

                let page=virt::search_first_unused_page().expect(PANIC_MSG);
                info!("Addr: {:X}",page.start_address().as_u64());
                let frame=PhysFrame::containing_address(phys_base);

                unsafe{PML4.lock().map_to(
                    page,frame,PageTableFlags::PRESENT,&mut *FRAME_MANAGER.lock()).expect(PANIC_MSG).flush()};

                let frame_offset=phys_base.as_u64()&0xfff;
                let base=page.start_address()+frame_offset;

                Self{
                    base
                }
            }

            #visibility fn get(&self,field:#enum_name)->#ty{
                let raw=self.get_raw();
                match field{
                    #(#enum_name::#enum_variants => raw.bit_range(#bit_range),)*
                }
            }

            #visibility fn set(&self,field:#enum_name,value:#ty){
                let mut raw=self.get_raw();
                match field{
                    #(#enum_name::#enum_variants => raw.set_bit_range(#bit_range,value),)*
                };

                unsafe{core::ptr::write(self.base.as_mut_ptr(),raw)}
            }

            fn get_raw(&self)->#ty{
                unsafe{core::ptr::read(self.base.as_mut_ptr())}
            }
        }

        impl core::ops::Drop for #name{
            fn drop(&mut self){
                use x86_64::structures::paging::{Page,FrameDeallocator,Mapper};

                let page= Page::containing_address(self.base);
                let (frame,flush)=crate::mem::paging::pml4::PML4.lock().unmap(page).unwrap();
                flush.flush();

                unsafe{crate::mem::allocator::phys::FRAME_MANAGER.lock().deallocate_frame(frame);}
            }
        }

        #[derive(Copy,Clone)]
        #visibility enum #enum_name{
            #(#enum_variants,)*
        }
    };

    TokenStream::from(expanded)
}
