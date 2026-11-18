//! Process-global plugin management, aligned with upstream test_plugin_management.py.

mod common;

use maa_framework::resource::Resource;

use common::init_test_env;

#[test]
fn test_shutdown_requires_destroyed_instances() {
    init_test_env().unwrap();

    maa_framework::get_plugin_list().unwrap();
    let resource = Resource::new().unwrap();
    assert!(maa_framework::shutdown_plugins().is_err());
    maa_framework::flush_plugins().unwrap();

    drop(resource);
    maa_framework::shutdown_plugins().unwrap();
    maa_framework::shutdown_plugins().unwrap();
    assert!(maa_framework::flush_plugins().is_err());
}
