//use libc::{SOCK_RAW};

//pub use libc::{
//    PF_CAN, CAN_RAW, SIOCGIFINDEX
//};

use std::{
    mem,
    os::{
        raw::{c_void}
    }
};

use nix::net::if_::if_nametoindex;

#[derive(Clone, Copy)]
struct CanAddr(libc::sockaddr_can);

impl CanAddr {

    pub fn new(ifindex: libc::c_uint) -> Self {
        let mut addr = Self::default();
        addr.0.can_ifindex = ifindex as libc::c_int;
        addr
    }

    pub fn as_ptr(&self) -> *const libc::sockaddr_can {
        &self.0
    }
}

impl Default for CanAddr {
    fn default() -> Self {
        let mut addr: libc::sockaddr_can = unsafe { mem::zeroed() };
        addr.can_family = libc::AF_CAN as libc::sa_family_t;
        Self(addr)
    }
}

impl From<libc::sockaddr_can> for CanAddr {
    fn from(addr: libc::sockaddr_can) -> Self {
        Self(addr)
    }
}

impl AsRef<libc::sockaddr_can> for CanAddr {
    fn as_ref(&self) -> &libc::sockaddr_can {
        &self.0
    }
}

bitflags::bitflags! {
    pub struct IdFlags: libc::canid_t {
        const EFF = libc::CAN_EFF_FLAG;
        const RTR = libc::CAN_RTR_FLAG;
        const ERR = libc::CAN_ERR_FLAG;
    }
}

fn init_id_word(id: libc::canid_t, mut flags: IdFlags) -> libc::canid_t {
    flags |= IdFlags::EFF;
    
    id | flags.bits()
}

#[derive(Clone, Copy)]
pub struct CanFrame(libc::can_frame);

impl CanFrame {
    pub fn init(id: u32, data: &[u8], flags: IdFlags) -> Self {
        let n = data.len();

        let mut frame: libc::can_frame = unsafe { mem::zeroed() };
        frame.can_id = init_id_word(id, flags);
        frame.can_dlc = n as u8;
        frame.data[..n].copy_from_slice(data);

        Self(frame)
    }

    pub fn as_ptr(&self) -> *const libc::can_frame {
        &self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut libc::can_frame {
        &mut self.0
    }
}

fn main() {
    // println!("Hello, world!");
    

    // open the socket connection
    let sock_fd = unsafe { libc::socket(libc::PF_CAN, libc::SOCK_RAW, libc::CAN_RAW) };
    
    if sock_fd < 0 {
        println!("Failed to open socket.");
        return ();
    }

    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::ioctl(sock_fd, libc::SIOCGIFINDEX, &mut ts as *mut libc::timespec); };

    let ifname = "vcan0";
    //let r = if_nametoindex(ifname);
    let ifindex: u32;
    match if_nametoindex(ifname) {
        Ok(val) => {
            ifindex = val;
        },
        Err(err) => {
            println!("{}", err);
            return ();
        }
    }
    let addr = CanAddr::new(ifindex);
    let bind_rval = unsafe {
        libc::bind(sock_fd,
                   addr.as_ptr() as *const libc::sockaddr,
                   mem::size_of::<CanAddr>() as u32)
    };
    if bind_rval == -1 {
        println!("Unable to bind socket!");
        unsafe { libc::close(sock_fd) };
        return ();
    }

    //let mut ts = timespec { tv_sec: 0, tv_nsec: 0 };
    //let rval = unsafe {
    //    libc::ioctl(sock_fd, SIOCGSTAMPNS as c_ulong, &mut ts as *mut timespec)
    //};

    let frame = CanFrame::init(0x123, &[], IdFlags::RTR);
    let frame_ptr = frame.as_ptr();
    let rval = unsafe { libc::write(sock_fd, frame_ptr as *const c_void, mem::size_of::<CanFrame>()) };

    if rval == -1 {
        return ();
    }

    // close the socket connection
    unsafe { libc::close(sock_fd) };
}
