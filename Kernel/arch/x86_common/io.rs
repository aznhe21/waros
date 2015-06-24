/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * arch/x86/x86_io.rs
 * - Support for the x86 IO bus
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/// Write a byte to the specified port
#[inline]
pub unsafe fn outb(port: u16, val: u8)
{
	asm!("outb %al, %dx" : : "{dx}"(port), "{al}"(val));
}

/// Read a single byte from the specified port
#[inline]
pub unsafe fn inb(port: u16) -> u8
{
	let ret : u8;
	asm!("inb %dx, %al" : "={al}"(ret) : "{dx}"(port));
	return ret;
}

/// Write a word (16-bits) to the specified port
#[inline]
pub unsafe fn outw(port: u16, val: u16)
{
	asm!("outw %ax, %dx" : : "{dx}"(port), "{ax}"(val));
}

/// Read a word (16-bits) from the specified port
#[inline]
pub unsafe fn inw(port: u16) -> u16
{
	let ret : u16;
	asm!("inw %dx, %ax" : "={ax}"(ret) : "{dx}"(port));
	return ret;
}

/// Write a long/double-word (32-bits) to the specified port
#[inline]
pub unsafe fn outl(port: u16, val: u32)
{
	asm!("outl %eax, %dx" : : "{dx}"(port), "{eax}"(val));
}

/// Read a long/double-word (32-bits) from the specified port
#[inline]
pub unsafe fn inl(port: u16) -> u32
{
	let ret : u32;
	asm!("inl %dx, %eax" : "={eax}"(ret) : "{dx}"(port));
	return ret;
}

#[inline(always)]
pub unsafe fn io_delay()
{
    let delay_port = 0x80_u16;
    asm!("outb %al, %dx" :: "{dx}"(delay_port) :: "volatile");
}

#[inline]
pub unsafe fn ds() -> u16
{
    let ret: u16;
    asm!("movw %ds, %ax" : "={ax}"(ret));
    return ret;
}

/*pub unsafe fn set_fs(val: u16)
{
    asm!("movw $0, %fs" :: "rm"(val) :: "volatile");
}

pub unsafe fn fs() -> u16
{
    let ret: u16;
    asm!("movw %fs, %ax" : "={ax}"(ret) ::: "volatile");
    return ret;
}

pub unsafe fn set_gs(val: u16)
{
    asm!("movw $0, %gs" :: "rm"(val) :: "volatile");
}

pub unsafe fn gs() -> u16
{
    let ret: u16;
    asm!("movw %gs, %ax" : "={ax}"(ret) ::: "volatile");
    return ret;
}

pub unsafe fn rdfs8(addr: u32) -> u8
{
    let ret: u8;
    asm!("movb %fs:$1, $0" : "=q"(ret) : "m"(*(addr as *mut u8)) :: "volatile");
    return ret;
}

pub unsafe fn rdfs16(addr: u32) -> u16
{
    let ret: u16;
    asm!("movw %fs:$1, $0" : "=r"(ret) : "m"(*(addr as *mut u16)) :: "volatile");
    return ret;
}

pub unsafe fn rdfs32(addr: u32) -> u32
{
    let ret: u32;
    asm!("movl %fs:$1, $0" : "=r"(ret) : "m"(*(addr as *mut u32)) :: "volatile");
    return ret;
}

pub unsafe fn wrfs8(addr: u32, val: u8)
{
    asm!("movb $1, %fs:$0" :: "m"(*(addr as *mut u8)), "qi"(val) :: "volatile");
}

pub unsafe fn wrfs16(addr: u32, val: u16)
{
    asm!("movw $1, %fs:$0" :: "m"(*(addr as *mut u16)), "ri"(val) :: "volatile");
}

pub unsafe fn wrfs32(addr: u32, val: u32)
{
    asm!("movl $1, %fs:$0" :: "m"(*(addr as *mut u32)), "ri"(val) :: "volatile");
}

pub unsafe fn rdgs8(addr: u32) -> u8
{
    let ret: u8;
    asm!("movb %gs:$1, $0" : "=q"(ret) : "m"(*(addr as *mut u8)) :: "volatile");
    return ret;
}

pub unsafe fn rdgs16(addr: u32) -> u16
{
    let ret: u16;
    asm!("movw %gs:$1, $0" : "=r"(ret) : "m"(*(addr as *mut u16)) :: "volatile");
    return ret;
}

pub unsafe fn rdgs32(addr: u32) -> u32
{
    let ret: u32;
    asm!("movl %gs:$1, $0" : "=r"(ret) : "m"(*(addr as *mut u32)) :: "volatile");
    return ret;
}

pub unsafe fn wrgs8(addr: u32, val: u8)
{
    asm!("movb $1, %gs:$0" :: "m"(*(addr as *mut u8)), "qi"(val) :: "volatile");
}

pub unsafe fn wrgs16(addr: u32, val: u16)
{
    asm!("movw $1, %gs:$0" :: "m"(*(addr as *mut u16)), "ri"(val) :: "volatile");
}

pub unsafe fn wrgs32(addr: u32, val: u32)
{
    asm!("movl $1, %gs:$0" :: "m"(*(addr as *mut u32)), "ri"(val) :: "volatile");
}*/

#[inline]
pub unsafe fn load_eflags() -> u32
{
    let ret: u32;
    asm!("pushf; pop %eax" : "={eax}"(ret) ::: "volatile");
    return ret;
}

#[inline]
pub unsafe fn save_eflags(eflags: u32)
{
    asm!("push %eax; popf" :: "{eax}"(eflags) :: "volatile");
}

