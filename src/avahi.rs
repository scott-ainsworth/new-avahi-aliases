//! A lightweight interface to Avahi via DBus
use std::time::Duration;

use crate::ErrorWrapper;

mod avahi_dbus;
use avahi_dbus::OrgFreedesktopAvahiServer;
mod interface;
pub use interface::Interface;
mod protocol;
pub use protocol::Protocol;
mod record_class;
pub use record_class::RecordClass;
mod record_type;
pub use record_type::RecordType;

const DBUS_NAME: &str = "org.freedesktop.Avahi";
const DBUS_PATH_SERVER: &str = "/";
const DBUS_INTERFACE_ENTRY_GROUP: &str = "org.freedesktop.Avahi.EntryGroup";
// const DBUS_INTERFACE_SERVER: &str = "org.freedesktop.Avahi.Server";


const DEFAULT_TTL: u64 = 60;

pub struct Avahi {
    conn: dbus::blocking::Connection,
    ttl: Duration,
}

impl<'a> Avahi {
    pub fn new() -> Result<Self, ErrorWrapper> {
        Ok(Self {
            conn: dbus::blocking::Connection::new_system()?,
            ttl: Duration::from_secs(DEFAULT_TTL),
        })
    }

    pub fn encode_rdata(name: &str) -> Vec<u8> {
        // TODO: convert encode_rdata to functional style
        let mut rdata: Vec<u8> = Vec::<u8>::new();
        for part in name.split('.').filter(|p| !p.is_empty()) {
            rdata.extend([part.len().to_be_bytes().last().unwrap()]);
            rdata.extend(to_ascii(part).as_bytes());
        }
        rdata.extend(&[0u8]);
        rdata
    }

    pub fn encode_name(name: &str) -> String {
        name.split('.')
            .filter(|p| !p.is_empty())
            .map(to_ascii)
            .collect::<Vec<String>>()
            .join(".")
    }

    pub fn get_group(&self) -> Result<AvahiGroup<'_>, ErrorWrapper> {
        let group_path = self.get_proxy().entry_group_new()?;
        let g = AvahiGroup::new(self, group_path, self.ttl);
        Ok(g)
    }

    pub fn get_host_name_fqdn(&self) -> Result<String, ErrorWrapper> {
        Ok(self.get_proxy().get_host_name_fqdn()?)
    }

    pub fn get_version(&self) -> Result<String, ErrorWrapper> {
        Ok(self.get_proxy().get_version_string()?)
    }

    fn get_proxy(&'a self) -> dbus::blocking::Proxy<'_, &dbus::blocking::Connection> {
        self.conn.with_proxy(DBUS_NAME, DBUS_PATH_SERVER, Duration::from_secs(5))
    }

    // pub fn add_record(&self, cname: &str, rdata: &[u8], ttl: u32) -> Result<(), ErrorWrapper>
    // {     let group = self.get_group()?;
    //     let record = AvahiRecord {
    //         interface: UNSPECIFIED_INTERFACE,
    //         protocol: PROTO_UNSPEC,
    //         res0: 0,
    //         name: cname,
    //         class: CLASS_IN,
    //         record_type: TYPE_CNAME,
    //         ttl: ttl,
    //         rdata: rdata,
    //     };
    //     let result = group.add_record(record)?;
    //     Ok(())
    // }

    // pub fn publish_alias(&self, alias: &Alias) -> Result<(), ErrorWrapper> {}
}

// struct AvahiRecord<'a> {
//     interface: i32,
//     protocol: i32,
//     name: &'a str,
//     class: i32,
//     record_type: i32,
//     ttl: u32,
//     rdata: &'a [u8],
// }

pub struct AvahiGroup<'a>(dbus::blocking::Proxy<'a, &'a dbus::blocking::Connection>);

impl<'a> AvahiGroup<'a> {
    fn new(avahi: &'a Avahi, path: dbus::Path<'a>, ttl: Duration) -> AvahiGroup<'a> {
        let g = AvahiGroup(avahi.conn.with_proxy(DBUS_NAME, path, ttl));
        g
    }

    pub fn add_record(&self, cname: &str, rdata: &[u8], ttl: u32) -> Result<(), ErrorWrapper> {
        let record = (
            Interface::Unspecified,
            Protocol::Unspecified,
            0u32,
            cname,
            RecordClass::In,
            RecordType::Cname,
            ttl,
            rdata,
        );
        self.0.method_call(DBUS_INTERFACE_ENTRY_GROUP, "AddRecord", record)?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), ErrorWrapper> {
        self.0.method_call(DBUS_INTERFACE_ENTRY_GROUP, "Commit", ())?;
        Ok(())
    }
}

/// Convert IDNA domains to ASCII (currently a no-op/passthrough)
fn to_ascii(idna_name: &str) -> String { idna_name.to_owned() }

#[cfg(test)]
mod test {

    use super::*;

    static TEST_DATA: &[(&str, &[u8])] = &[
        ("a0.local", &[2, b'a', b'0', 5, b'l', b'o', b'c', b'a', b'l', 0]),
        ("xyzzy.local", &[5, b'x', b'y', b'z', b'z', b'y', 5, b'l', b'o', b'c', b'a', b'l', 0]),
    ];

    #[test]
    fn dbus_constants_are_correct() {
        assert_eq!(DBUS_NAME, "org.freedesktop.Avahi");
        // assert_eq!(DBUS_INTERFACE_SERVER, "org.freedesktop.avahi.server");
        assert_eq!(DBUS_PATH_SERVER, "/");
        assert_eq!(DBUS_INTERFACE_ENTRY_GROUP, "org.freedesktop.Avahi.EntryGroup");
        assert_eq!(Interface::Unspecified as i32, -1);
        assert_eq!(RecordClass::In as u32, 0x01);
        assert_eq!(RecordType::Cname as u32, 0x05);
        assert_eq!(Protocol::Unspecified as i32, -1);
        assert_eq!(DEFAULT_TTL, 60);
    }

    #[test]
    fn dbus_creation_succeeds() -> Result<(), ErrorWrapper> {
        let avahi = Avahi::new()?;
        // let proxy = avahi.get_proxy();
        eprintln!("**** avahi.get_version() = {}", avahi.get_version()?);
        eprintln!("**** avahi.get_host_name_fqdn() = {}", avahi.get_host_name_fqdn()?);
        Ok(())
    }

    #[test]
    fn resource_records_are_created_correctly() {
        for (alias, resource_record) in TEST_DATA {
            assert_eq!(*resource_record, Avahi::encode_rdata(alias).as_slice());
        }
    }
}
