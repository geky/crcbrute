/// Hardware accelerated carry-less multiplication

#[inline]
pub fn pmul64(a: u64, b: u64) -> (u64, u64) {
    #[cfg(all(
        not(feature="sw-pmul"),
        target_arch="x86_64",
        target_feature="pclmulqdq"
    ))]
    {
        // x86_64 provides 64-bit xmul via the pclmulqdq instruction
        use core::arch::x86_64::*;
        unsafe {
            let a = _mm_set_epi64x(0, a as i64);
            let b = _mm_set_epi64x(0, b as i64);
            let x = _mm_clmulepi64_si128::<0>(a, b);
            let lo = _mm_extract_epi64::<0>(x) as u64;
            let hi = _mm_extract_epi64::<1>(x) as u64;
            (lo, hi)
        }
    }

    #[cfg(all(
        not(feature="sw-pmul"),
        target_arch="aarch64",
        target_feature="neon"
    ))]
    {
        // aarch64 provides 64-bit xmul via the pmull instruction
        use core::arch::aarch64::*;
        unsafe {
            let x = vmull_p64(a as u64, b as u64);
            (x as u64, (x >> 64) as u64)
        }
    }

    #[cfg(all(
        not(feature="hw-pmul"),
        not(all(
            not(feature="sw-pmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq")),
        not(all(
            not(feature="sw-pmul"),
            target_arch="aarch64",
            target_feature="neon")),
    ))]
    {
        let mut lo = 0;
        let mut hi = 0;
        let mut i = 0;
        while i < 64 {
            let mask = (((a as i64) << (64-1-i)) >> (64-1)) as u64;
            lo ^= mask & (b << i);
            hi ^= mask & (b >> (64-1-i));
            i += 1;
        }
        // note we adjust hi by one here to avoid handlings shifts > word size
        (lo, hi >> 1)
    }
}

#[inline]
pub fn pmul32(a: u32, b: u32) -> (u32, u32) {
    let (lo, _) = pmul64(a as u64, b as u64);
    (lo as u32, (lo >> 32) as u32)
}
