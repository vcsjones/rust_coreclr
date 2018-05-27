extern crate libloading as lib;

use std::collections::{HashMap};
use std::ffi::{CString};
use std::io;
use std::io::{Error};
use std::mem;
use std::os::raw::{c_int, c_char, c_uint, c_void};
use std::ptr;
use std::vec::{Vec};

type CoreCLRInitialize = extern "C" fn (*const c_char, *const c_char, c_int, *const *mut c_char, *const *mut c_char, *mut *mut c_void, *mut c_uint) -> c_int;
type CoreCLRCreateDelegate = extern "C" fn(*mut c_void, c_uint, *const c_char, *const c_char, *const c_char, *mut *mut c_void) -> c_int;

pub struct CoreCLR {
    library: lib::Library
}

#[derive(Debug)]
pub struct Runtime {
    domain_id : c_uint,
    host_handle : *mut c_void
}

impl CoreCLR {
    pub fn new(library_path : &str) -> io::Result<CoreCLR> {
        let library = lib::Library::new(library_path)?;
        Ok(CoreCLR {
            library: library
        })
    }

    pub fn coreclr_createdelegate(&self, runtime : &Runtime, assembly_name : &str, type_name : &str, method_name : &str) -> io::Result<fn() -> ()> {
        let create_delegate: lib::Symbol<CoreCLRCreateDelegate> = unsafe { self.library.get(b"coreclr_create_delegate")? };
        let assembly_name_ptr = CString::new(assembly_name)?;
        let type_name_ptr = CString::new(type_name)?;
        let method_name_ptr = CString::new(method_name)?;
        let mut delegate_handle = ptr::null_mut();
        let result = create_delegate(
            runtime.host_handle,
            runtime.domain_id,
            assembly_name_ptr.as_ptr(),
            type_name_ptr.as_ptr(),
            method_name_ptr.as_ptr(),
            &mut delegate_handle
        );
        if result < 0 {
            Result::Err(Error::from_raw_os_error(result))
        }
        else {
            Ok(unsafe { mem::transmute::<*mut c_void, fn() -> ()>(delegate_handle) })
        }
    }

    pub fn coreclr_initialize(&self, exe_path : &str, app_domain_friendly_name : &str, properties : HashMap<&str, &str>) -> io::Result<Runtime> {
        let exe_path_ptr = CString::new(exe_path)?;
        let app_domain_friendly_name_ptr = CString::new(app_domain_friendly_name)?;
        let mut keys = Vec::with_capacity(properties.len());
        let mut values = Vec::with_capacity(properties.len());
        for (&key, &value) in properties.iter() {
            keys.push(CString::new(key).unwrap().into_raw());
            values.push(CString::new(value).unwrap().into_raw());
        };
        let mut host_handle = ptr::null_mut();
        let mut domain_id = 0;
        let initializer: lib::Symbol<CoreCLRInitialize> = unsafe { self.library.get(b"coreclr_initialize")? };
        let result = initializer(
            exe_path_ptr.as_ptr(),
            app_domain_friendly_name_ptr.as_ptr(),
            properties.len() as i32,
            keys.as_ptr(),
            values.as_ptr(),
            &mut host_handle,
            &mut domain_id
        );

        // We need to call from_raw on our original C strings because into_raw
        // forgets them. If we don't, we leak the memory in keys and values.
        for key in keys.into_iter() {
            let _ = unsafe { CString::from_raw(key) };
        };
        for value in values.into_iter() {
            let _ = unsafe { CString::from_raw(value) };
        };
        if result < 0 {
            Result::Err(Error::from_raw_os_error(result))
        }
         else {
            Result::Ok(Runtime { domain_id : domain_id, host_handle : host_handle })
        }
    }
}
