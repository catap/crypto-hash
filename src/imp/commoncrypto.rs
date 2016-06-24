// Copyright (c) 2016 Mark Lee
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! A cryptographic hash generator dependent upon OSX's `CommonCrypto`.

use libc::{c_int, c_uint, c_ulong, c_ulonglong};
use std::io;
use super::Algorithm;

macro_rules! unsafe_guard {
    ($e: expr) => {
        unsafe {
            let r = $e;
            assert_eq!(r, 1);
        }
    }
}

const MD5_CBLOCK: usize = 64;
const MD5_LBLOCK: usize = MD5_CBLOCK / 4;
const MD5_DIGEST_LENGTH: usize = 16;

const SHA_LBLOCK: usize = 16;
const SHA256_DIGEST_LENGTH: usize = 32;
const SHA512_DIGEST_LENGTH: usize = 64;

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
struct CC_MD5_CTX {
    A: c_uint,
    B: c_uint,
    C: c_uint,
    D: c_uint,
    Nl: c_uint,
    Nh: c_uint,
    data: [c_uint; MD5_LBLOCK],
    num: c_uint,
}

impl CC_MD5_CTX {
    fn new() -> CC_MD5_CTX {
        CC_MD5_CTX {
            A: 0,
            B: 0,
            C: 0,
            D: 0,
            Nl: 0,
            Nh: 0,
            data: [0 as c_uint; MD5_LBLOCK],
            num: 0,
        }
    }
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
struct CC_SHA256_CTX {
    h: [c_ulong; 8],
    Nl: c_ulong,
    Nh: c_ulong,
    data: [c_ulong; SHA_LBLOCK],
    num: c_uint,
    md_len: c_uint,
}

impl CC_SHA256_CTX {
    fn new() -> CC_SHA256_CTX {
        CC_SHA256_CTX {
            h: [0 as c_ulong; 8],
            Nl: 0,
            Nh: 0,
            data: [0 as c_ulong; SHA_LBLOCK],
            num: 0,
            md_len: 0,
        }
    }
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
struct CC_SHA512_CTX {
    h: [c_ulonglong; 8],
    Nl: c_ulonglong,
    Nh: c_ulonglong,
    data: [c_ulonglong; SHA_LBLOCK],
    num: c_uint,
    md_len: c_uint,
}

impl CC_SHA512_CTX {
    fn new() -> CC_SHA512_CTX {
        CC_SHA512_CTX {
            h: [0 as c_ulonglong; 8],
            Nl: 0,
            Nh: 0,
            data: [0 as c_ulonglong; SHA_LBLOCK],
            num: 0,
            md_len: 0,
        }
    }
}

#[derive(Debug)]
enum DigestContext {
    MD5(CC_MD5_CTX),
    SHA256(CC_SHA256_CTX),
    SHA512(CC_SHA512_CTX),
}

extern "C" {
    fn CC_MD5_Init(ctx: *mut CC_MD5_CTX) -> c_int;
    fn CC_MD5_Update(ctx: *mut CC_MD5_CTX, data: *const u8, n: usize) -> c_int;
    fn CC_MD5_Final(md: *mut u8, ctx: *mut CC_MD5_CTX) -> c_int;
    fn CC_SHA256_Init(ctx: *mut CC_SHA256_CTX) -> c_int;
    fn CC_SHA256_Update(ctx: *mut CC_SHA256_CTX, data: *const u8, n: usize) -> c_int;
    fn CC_SHA256_Final(md: *mut u8, ctx: *mut CC_SHA256_CTX) -> c_int;
    fn CC_SHA512_Init(ctx: *mut CC_SHA512_CTX) -> c_int;
    fn CC_SHA512_Update(ctx: *mut CC_SHA512_CTX, data: *const u8, n: usize) -> c_int;
    fn CC_SHA512_Final(md: *mut u8, ctx: *mut CC_SHA512_CTX) -> c_int;
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum State {
    Reset,
    Updated,
    Finalized,
}

#[derive(Debug)]
pub struct Hasher {
    context: DigestContext,
    state: State,
}

fn md5_new() -> CC_MD5_CTX {
    let mut ctx: CC_MD5_CTX = CC_MD5_CTX::new();
    md5_init(&mut ctx);
    ctx
}

fn md5_init(ctx: &mut CC_MD5_CTX) {
    unsafe_guard!(CC_MD5_Init(ctx));
    assert!(!(ctx as *mut CC_MD5_CTX).is_null());
}

fn md5_write(ctx: &mut CC_MD5_CTX, buf: &[u8]) {
    unsafe_guard!(CC_MD5_Update(ctx, buf.as_ptr(), buf.len()));
}

fn md5_finish(ctx: &mut CC_MD5_CTX) -> Vec<u8> {
    let mut md = [0u8; MD5_DIGEST_LENGTH];
    unsafe_guard!(CC_MD5_Final(md.as_mut_ptr(), ctx));
    md.to_vec()
}

fn sha256_new() -> CC_SHA256_CTX {
    let mut ctx: CC_SHA256_CTX = CC_SHA256_CTX::new();
    sha256_init(&mut ctx);
    ctx
}

fn sha512_new() -> CC_SHA512_CTX {
    let mut ctx: CC_SHA512_CTX = CC_SHA512_CTX::new();
    sha512_init(&mut ctx);
    ctx
}

fn sha256_init(ctx: &mut CC_SHA256_CTX) {
    unsafe_guard!(CC_SHA256_Init(ctx));
    assert!(!(ctx as *mut CC_SHA256_CTX).is_null());
}

fn sha512_init(ctx: &mut CC_SHA512_CTX) {
    unsafe_guard!(CC_SHA512_Init(ctx));
    assert!(!(ctx as *mut CC_SHA512_CTX).is_null());
}

fn sha256_write(ctx: &mut CC_SHA256_CTX, buf: &[u8]) {
    unsafe_guard!(CC_SHA256_Update(ctx, buf.as_ptr(), buf.len()));
}

fn sha512_write(ctx: &mut CC_SHA512_CTX, buf: &[u8]) {
    unsafe_guard!(CC_SHA512_Update(ctx, buf.as_ptr(), buf.len()));
}

fn sha256_finish(ctx: &mut CC_SHA256_CTX) -> Vec<u8> {
    let mut md = [0u8; SHA256_DIGEST_LENGTH];
    unsafe_guard!(CC_SHA256_Final(md.as_mut_ptr(), ctx));
    md.to_vec()
}

fn sha512_finish(ctx: &mut CC_SHA512_CTX) -> Vec<u8> {
    let mut md = [0u8; SHA512_DIGEST_LENGTH];
    unsafe_guard!(CC_SHA512_Final(md.as_mut_ptr(), ctx));
    md.to_vec()
}

impl Hasher {
    pub fn new(algorithm: Algorithm) -> Hasher {
        let context = match algorithm {
            Algorithm::MD5 => DigestContext::MD5(md5_new()),
            Algorithm::SHA256 => DigestContext::SHA256(sha256_new()),
            Algorithm::SHA512 => DigestContext::SHA512(sha512_new()),
        };

        Hasher {
            context: context,
            state: State::Reset,
        }
    }

    fn init(&mut self) {
        match self.state {
            State::Reset => return,
            State::Updated => {
                self.finish();
            }
            State::Finalized => (),
        }
        match self.context {
            DigestContext::MD5(ref mut ctx) => md5_init(ctx),
            DigestContext::SHA256(ref mut ctx) => sha256_init(ctx),
            DigestContext::SHA512(ref mut ctx) => sha512_init(ctx),
        }
        self.state = State::Reset;
    }

    pub fn finish(&mut self) -> Vec<u8> {
        if self.state == State::Finalized {
            self.init();
        }
        let result = match self.context {
            DigestContext::MD5(ref mut ctx) => md5_finish(ctx),
            DigestContext::SHA256(ref mut ctx) => sha256_finish(ctx),
            DigestContext::SHA512(ref mut ctx) => sha512_finish(ctx),
        };
        self.state = State::Finalized;

        result
    }
}

impl io::Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.state == State::Finalized {
            self.init();
        }
        match self.context {
            DigestContext::MD5(ref mut ctx) => md5_write(ctx, buf),
            DigestContext::SHA256(ref mut ctx) => sha256_write(ctx, buf),
            DigestContext::SHA512(ref mut ctx) => sha512_write(ctx, buf),
        }
        self.state = State::Updated;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Hasher {
    fn drop(&mut self) {
        if self.state != State::Finalized {
            self.finish();
        }
    }
}
