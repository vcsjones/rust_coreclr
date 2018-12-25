mod coreclr;

use std::collections::{HashMap};


fn main() {
    
    let exe_path = "<path to the container folder of your assemblies>";
    let app_domain = "rusty";
    let assembly_name = "MyAssembly";
    let type_name = "MyAssembly.MyType";
    let entry_point = "MyEntryPointMethod";
    let mut map = HashMap::new();
    map.insert("APP_PATHS", exe_path);
    map.insert("TRUSTED_PLATFORM_ASSEMBLIES", "<list of trusted platform assemblies>");
    let coreclr = coreclr::CoreCLR::new("/the/path/to/libcoreclr.dylib", exe_path, app_domain, &map).unwrap();
    let managed_function : coreclr::ManagedDelegate<unsafe extern "C" fn() -> ()> = coreclr.create_delegate(assembly_name, type_name, entry_point).unwrap();
    unsafe { managed_function() };
}
